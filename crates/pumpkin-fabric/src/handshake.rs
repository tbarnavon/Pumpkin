//! The configuration-phase handshake Fabric's server runs, as a state machine without I/O.
//!
//! Order, from fabric-networking-api-v1 `ServerConfigurationNetworkAddon.startConfiguration`
//! (line 92), `ServerConfigurationPacketListenerImplMixin.onClientReady`, and
//! `CommonPacketsImpl.init` (line 72):
//!
//! 1. Server sends `minecraft:register` (its configuration channels) and Ping `0xFAB71C`, then
//!    waits. A `minecraft:register` back means a Fabric client; the pong arriving first means a
//!    vanilla client.
//! 2. Early tasks (`BEFORE_CONFIGURE`): `fabric:registry/sync/direct`, waiting for
//!    `fabric:registry/sync/complete` (`RegistrySyncManager.configureClient`).
//! 3. `CONFIGURE` tasks: `c:version`, then `c:register` with the server's play channels, each
//!    waiting for the client's answer.
//! 4. The vanilla configuration continues (brand, known packs, registries, tags, finish).

use crate::sync_map;
use crate::wire::{common, register, registry_sync};

/// `ServerConfigurationNetworkAddon.startConfiguration` pings with this id (line 96).
pub const PING_ID: i32 = 0x00FA_B71C;

/// Configuration channels this server can receive, announced in step 1.
pub const SERVER_CONFIG_CHANNELS: &[&str] = &[
    common::VERSION_CHANNEL,
    common::REGISTER_CHANNEL,
    registry_sync::SYNC_COMPLETE_CHANNEL,
];

/// Something to send to the client.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Outgoing {
    Payload {
        channel: &'static str,
        data: Vec<u8>,
    },
    Ping(i32),
}

/// What the connection should do after feeding the handshake a packet.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Step {
    /// Send these and keep waiting.
    Send(Vec<Outgoing>),
    /// The packet was not part of the handshake; handle it as usual.
    NotHandled,
    /// Handshake packet consumed, nothing to send yet.
    Wait,
    /// The Fabric part is done: run the normal configuration.
    Done(Vec<Outgoing>),
    /// Disconnect the client with this message.
    Disconnect(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum State {
    AwaitingRegister,
    AwaitingSyncComplete,
    AwaitingVersion,
    AwaitingCommonRegister,
    Done,
}

/// Per-connection handshake. Only created when mods are installed; vanilla servers never run it.
#[derive(Debug)]
pub struct FabricHandshake {
    state: State,
    /// Mod namespaces, for the disconnect message shown to clients without them.
    namespaces: Vec<String>,
    /// Play channels this server receives, announced through `c:register`.
    server_play_channels: Vec<String>,
    client_config_channels: Vec<String>,
    client_play_channels: Vec<String>,
    common_version: Option<i32>,
}

impl FabricHandshake {
    #[must_use]
    pub const fn new(namespaces: Vec<String>, server_play_channels: Vec<String>) -> Self {
        Self {
            state: State::AwaitingRegister,
            namespaces,
            server_play_channels,
            client_config_channels: Vec::new(),
            client_play_channels: Vec::new(),
            common_version: None,
        }
    }

    /// Packets to send right after login acknowledgement, before anything vanilla.
    #[must_use]
    pub fn start(&self) -> Vec<Outgoing> {
        vec![
            Outgoing::Payload {
                channel: register::REGISTER_CHANNEL,
                data: register::encode(SERVER_CONFIG_CHANNELS),
            },
            Outgoing::Ping(PING_ID),
        ]
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

    /// Configuration channels the client said it can receive (from its `minecraft:register`).
    #[must_use]
    pub fn client_config_channels(&self) -> &[String] {
        &self.client_config_channels
    }

    fn missing_mods_message(&self) -> String {
        // Wording follows RegistrySyncManager.getIncompatibleClientComponent (line 96).
        format!(
            "This server requires Fabric Loader and Fabric API installed on your client!\n\
             The following mods are required: {}",
            self.namespaces.join(", ")
        )
    }

    /// Feed a configuration-phase pong.
    pub fn on_pong(&mut self, id: i32) -> Step {
        if id != PING_ID {
            return Step::NotHandled;
        }
        if self.state == State::AwaitingRegister {
            // The pong beat any registration packet: a vanilla client.
            return Step::Disconnect(self.missing_mods_message());
        }
        Step::Wait
    }

    /// Feed a configuration-phase custom payload from the client.
    pub fn on_payload(&mut self, channel: &str, data: &[u8]) -> Step {
        match (self.state, channel) {
            (_, register::REGISTER_CHANNEL) => {
                self.client_config_channels.extend(register::decode(data));
                if self.state != State::AwaitingRegister {
                    return Step::Wait;
                }
                if !self
                    .client_config_channels
                    .iter()
                    .any(|c| c == registry_sync::SYNC_CHANNEL)
                {
                    // Fabric Loader without Fabric API's registry sync, or another loader.
                    return Step::Disconnect(self.missing_mods_message());
                }
                self.state = State::AwaitingSyncComplete;
                let body = registry_sync::encode(&sync_map::build());
                Step::Send(
                    registry_sync::into_payloads(&body)
                        .into_iter()
                        .map(|data| Outgoing::Payload {
                            channel: registry_sync::SYNC_CHANNEL,
                            data,
                        })
                        .collect(),
                )
            }
            (State::AwaitingSyncComplete, registry_sync::SYNC_COMPLETE_CHANNEL) => {
                self.state = State::AwaitingVersion;
                Step::Send(vec![Outgoing::Payload {
                    channel: common::VERSION_CHANNEL,
                    data: common::encode_version(common::SUPPORTED_VERSIONS),
                }])
            }
            (State::AwaitingVersion, common::VERSION_CHANNEL) => {
                let Some(version) = common::decode_version(data)
                    .ok()
                    .and_then(|versions| common::negotiate(&versions))
                else {
                    return Step::Disconnect(String::from(
                        "Unsupported Fabric networking version (c:version)",
                    ));
                };
                self.common_version = Some(version);
                self.state = State::AwaitingCommonRegister;
                let channels: Vec<&str> = self
                    .server_play_channels
                    .iter()
                    .map(String::as_str)
                    .collect();
                Step::Send(vec![Outgoing::Payload {
                    channel: common::REGISTER_CHANNEL,
                    data: common::encode_register(version, common::PLAY_PROTOCOL, &channels),
                }])
            }
            (State::AwaitingCommonRegister, common::REGISTER_CHANNEL) => {
                let Ok(payload) = common::decode_register(data) else {
                    return Step::Disconnect(String::from("Malformed c:register payload"));
                };
                // CommonPacketsImpl.init (line 58): the version must match the negotiated one.
                if Some(payload.version) != self.common_version {
                    return Step::Disconnect(String::from("c:register version mismatch"));
                }
                if payload.protocol == common::PLAY_PROTOCOL {
                    self.client_play_channels = payload.channels;
                } else {
                    self.client_config_channels.extend(payload.channels);
                }
                self.state = State::Done;
                Step::Done(Vec::new())
            }
            _ => Step::NotHandled,
        }
    }
}
