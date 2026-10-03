//! The configuration-phase handshake NeoForge's server runs, as a state machine without I/O.
//!
//! Order, from the `ServerConfigurationPacketListenerImpl` patch, `NetworkRegistry` and
//! `ConfigurationInitialization`:
//!
//! 1. The server sends `minecraft:register`, an empty `neoforge:register` query and a ping. A
//!    NeoForge client answers the query with its channels before the pong. (Detecting the
//!    loader is the caller's job, since Fabric clients answer `minecraft:register` too.)
//! 2. `initializeNeoForgeConnection`: both sides' channels are negotiated per protocol. On
//!    failure the client is disconnected; on success the server sends `neoforge:network` with
//!    the agreed channels and `minecraft:register` with the channels it now listens on.
//! 3. Early task `SyncRegistries`: `frozen_registry_sync_start`, one `frozen_registry` per
//!    registry, `frozen_registry_sync_completed`, answered by the client's completion.
//! 4. `CommonVersionTask` and `CommonRegisterTask` (`c:version`, `c:register`), when the client
//!    listens on them, each waiting for the client's answer.
//! 5. `SyncConfig`: each synced config file as `neoforge:config_file`, without an answer.
//! 6. The vanilla configuration continues.

use pumpkin_fabric::handshake::{Outgoing, Step};
use pumpkin_fabric::wire::{common, register, registry_sync::SyncedRegistry};

use crate::negotiation::negotiate;
use crate::wire::{self, Component, Flow, Protocol};

/// Version NeoForge registers its own channels with (`NetworkInitialization`, `registrar("1")`).
const NEOFORGE_CHANNEL_VERSION: &str = "1";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum State {
    AwaitingSyncCompleted,
    AwaitingVersion,
    AwaitingCommonRegister,
    Done,
}

/// The channels this server registers, like NeoForge's `PAYLOAD_REGISTRATIONS`: the ones the
/// handshake uses, all optional so clients without them can still join.
#[must_use]
pub fn server_channels() -> Vec<(Protocol, Vec<Component>)> {
    let channel = |id: &str, flow| Component {
        id: id.to_string(),
        version: NEOFORGE_CHANNEL_VERSION.to_string(),
        flow,
        optional: true,
    };
    vec![
        (
            Protocol::Configuration,
            vec![
                channel(wire::CONFIG_FILE_CHANNEL, Some(Flow::Clientbound)),
                channel(wire::SYNC_START_CHANNEL, Some(Flow::Clientbound)),
                channel(wire::SYNC_CHANNEL, Some(Flow::Clientbound)),
                channel(wire::SYNC_COMPLETED_CHANNEL, None),
            ],
        ),
        (
            Protocol::Play,
            vec![channel(wire::CONFIG_FILE_CHANNEL, Some(Flow::Clientbound))],
        ),
    ]
}

/// Per-connection handshake, created once the client is known to run NeoForge.
#[derive(Debug)]
pub struct NeoForgeHandshake {
    state: State,
    server_channels: Vec<(Protocol, Vec<Component>)>,
    registries: Vec<SyncedRegistry>,
    /// Synced config files (name, TOML contents), sent when the client has `config_file`.
    config_files: Vec<(String, Vec<u8>)>,
    sync_config: bool,
    /// Whether the client listens on the `c:` channels (from its `minecraft:register`).
    common_channels: bool,
    common_version: Option<i32>,
    client_play_channels: Vec<String>,
}

impl NeoForgeHandshake {
    /// `registries` are the ones to sync: every registry a mod added entries to, with all of
    /// its entries. `config_files` are the synced configs, NeoForge's own included.
    #[must_use]
    pub const fn new(
        server_channels: Vec<(Protocol, Vec<Component>)>,
        registries: Vec<SyncedRegistry>,
        config_files: Vec<(String, Vec<u8>)>,
    ) -> Self {
        Self {
            state: State::AwaitingSyncCompleted,
            server_channels,
            registries,
            config_files,
            sync_config: false,
            common_channels: false,
            common_version: None,
            client_play_channels: Vec::new(),
        }
    }

    #[must_use]
    pub fn is_done(&self) -> bool {
        self.state == State::Done
    }

    /// Play channels the client said it can receive (from its `c:register`).
    #[must_use]
    pub fn client_play_channels(&self) -> &[String] {
        &self.client_play_channels
    }

    /// Steps 2 and 3, given the client's `neoforge:register` query and the channels from its
    /// `minecraft:register`.
    pub fn on_detected(&mut self, query: &[u8], client_listens_on: &[String]) -> Step {
        let Ok(client) = wire::decode_query(query) else {
            return Step::Disconnect(String::from("Malformed neoforge:register payload"));
        };
        let mut setup = Vec::new();
        let mut failures = Vec::new();
        for (protocol, server) in &self.server_channels {
            let client_side = client
                .iter()
                .find(|(p, _)| p == protocol)
                .map_or(&[][..], |(_, components)| components.as_slice());
            match negotiate(server, client_side) {
                Ok(agreed) => setup.push((*protocol, agreed)),
                Err(reasons) => failures.extend(reasons),
            }
        }
        if !failures.is_empty() {
            // `initializeNeoForgeConnection` also sends `modded_network_setup_failed` so the
            // client's screen lists each channel; the disconnect message carries the same list.
            let lines: Vec<String> = failures
                .iter()
                .map(|(id, reason)| format!("{id}: {reason}"))
                .collect();
            return Step::Disconnect(format!(
                "This server and your mods don't match:\n{}",
                lines.join("\n")
            ));
        }

        self.sync_config = setup.iter().any(|(protocol, channels)| {
            *protocol == Protocol::Configuration
                && channels
                    .iter()
                    .any(|(id, _)| id == wire::CONFIG_FILE_CHANNEL)
        });
        self.common_channels = client_listens_on
            .iter()
            .any(|c| c == common::VERSION_CHANNEL);
        let mut listening: Vec<&str> = wire::BUILTIN_CHANNELS.to_vec();
        for (protocol, channels) in &setup {
            if *protocol == Protocol::Configuration {
                listening.extend(channels.iter().map(|(id, _)| id.as_str()));
            }
        }
        let mut out = vec![
            Outgoing::Payload {
                channel: wire::NETWORK_CHANNEL,
                data: wire::encode_setup(&setup),
            },
            Outgoing::Payload {
                channel: register::REGISTER_CHANNEL,
                data: register::encode(&listening),
            },
        ];
        let names: Vec<&str> = self.registries.iter().map(|r| r.id.as_str()).collect();
        out.push(Outgoing::Payload {
            channel: wire::SYNC_START_CHANNEL,
            data: wire::encode_sync_start(&names),
        });
        for registry in &self.registries {
            out.push(Outgoing::Payload {
                channel: wire::SYNC_CHANNEL,
                data: wire::encode_registry(&registry.id, &registry.entries),
            });
        }
        out.push(Outgoing::Payload {
            channel: wire::SYNC_COMPLETED_CHANNEL,
            data: Vec::new(),
        });
        self.state = State::AwaitingSyncCompleted;
        Step::Send(out)
    }

    /// Step 5: the synced config files, if the client agreed to `config_file`.
    fn config_payloads(&self) -> Vec<Outgoing> {
        if !self.sync_config {
            return Vec::new();
        }
        self.config_files
            .iter()
            .map(|(name, contents)| Outgoing::Payload {
                channel: wire::CONFIG_FILE_CHANNEL,
                data: wire::encode_config_file(name, contents),
            })
            .collect()
    }

    /// Feed a configuration-phase custom payload from the client, after `on_detected`.
    pub fn on_payload(&mut self, channel: &str, data: &[u8]) -> Step {
        match (self.state, channel) {
            (State::AwaitingSyncCompleted, wire::SYNC_COMPLETED_CHANNEL) => {
                if !self.common_channels {
                    self.state = State::Done;
                    return Step::Done(self.config_payloads());
                }
                self.state = State::AwaitingVersion;
                Step::Send(vec![Outgoing::Payload {
                    channel: common::VERSION_CHANNEL,
                    data: common::encode_version(common::SUPPORTED_VERSIONS),
                }])
            }
            (State::AwaitingVersion, common::VERSION_CHANNEL) => {
                // `NetworkRegistry.checkCommonVersion`.
                let Some(version) = common::decode_version(data)
                    .ok()
                    .and_then(|versions| common::negotiate(&versions))
                else {
                    return Step::Disconnect(String::from(
                        "Unsupported common network version (c:version)",
                    ));
                };
                self.common_version = Some(version);
                self.state = State::AwaitingCommonRegister;
                Step::Send(vec![Outgoing::Payload {
                    channel: common::REGISTER_CHANNEL,
                    data: common::encode_register(version, common::PLAY_PROTOCOL, &[]),
                }])
            }
            (State::AwaitingCommonRegister, common::REGISTER_CHANNEL) => {
                let Ok(payload) = common::decode_register(data) else {
                    return Step::Disconnect(String::from("Malformed c:register payload"));
                };
                if payload.protocol == common::PLAY_PROTOCOL {
                    self.client_play_channels = payload.channels;
                }
                self.state = State::Done;
                Step::Done(self.config_payloads())
            }
            // The client's answer to the server's `minecraft:register`.
            (_, register::REGISTER_CHANNEL) => Step::Wait,
            _ => Step::NotHandled,
        }
    }
}

#[cfg(test)]
#[allow(clippy::panic)]
mod tests {
    use super::*;
    use pumpkin_fabric::wire::write_string;

    /// A plain NeoForge client's query: its own configuration channels, all optional.
    fn plain_client_query() -> Vec<u8> {
        let mut query = vec![1, Protocol::Configuration as u8, 2];
        for (id, flow) in [
            (wire::SYNC_CHANNEL, Some(1u8)),
            (wire::SYNC_COMPLETED_CHANNEL, None),
        ] {
            write_string(&mut query, id);
            write_string(&mut query, "1");
            match flow {
                Some(flow) => query.extend([1, flow]),
                None => query.push(0),
            }
            query.push(1);
        }
        query
    }

    #[test]
    fn a_plain_client_negotiates_and_syncs() {
        let mut handshake = NeoForgeHandshake::new(server_channels(), Vec::new(), Vec::new());
        let Step::Send(out) =
            handshake.on_detected(&plain_client_query(), &["c:version".to_string()])
        else {
            panic!("expected packets");
        };
        let channels: Vec<&str> = out
            .iter()
            .map(|o| match o {
                Outgoing::Payload { channel, .. } => *channel,
                Outgoing::Ping(_) => "ping",
            })
            .collect();
        assert_eq!(
            channels,
            [
                wire::NETWORK_CHANNEL,
                "minecraft:register",
                wire::SYNC_START_CHANNEL,
                wire::SYNC_COMPLETED_CHANNEL
            ]
        );
        assert!(matches!(
            handshake.on_payload(wire::SYNC_COMPLETED_CHANNEL, &[]),
            Step::Send(_)
        ));
    }

    #[test]
    fn a_required_mod_channel_disconnects() {
        let mut query = vec![1, Protocol::Play as u8, 1];
        write_string(&mut query, "somemod:data");
        write_string(&mut query, "1");
        query.extend([0, 0]);
        let mut handshake = NeoForgeHandshake::new(server_channels(), Vec::new(), Vec::new());
        assert!(matches!(
            handshake.on_detected(&query, &[]),
            Step::Disconnect(_)
        ));
    }
}
