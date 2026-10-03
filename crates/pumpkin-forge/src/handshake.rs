//! The configuration tasks Forge's server runs with a Forge client, as a state machine without
//! I/O.
//!
//! Order, from `ForgeNetworkConfigurationHandler.gatherInit` (all on `forge:handshake` except
//! the first):
//!
//! 1. `RegisterChannelsTask`: `minecraft:register` with the server's channels; no answer.
//! 2. `ModVersionsTask`: `ModVersions`, answered by the client's `ModVersions`.
//! 3. `ChannelVersionsTask`: `ChannelVersions`, answered by the client's `ChannelVersions`
//!    (or `MismatchData` and a disconnect when the client rejects the server's channels).
//! 4. `SyncRegistriesTask`: `RegistryList`, then one `RegistryData` per registry, each sent
//!    once the client acknowledged the previous message's token.
//! 5. `SyncConfigTask`: one `ConfigData` per server config; no answer.
//! 6. The vanilla configuration continues.

use pumpkin_fabric::handshake::{Outgoing, Step};
use pumpkin_fabric::wire::{register, registry_sync::SyncedRegistry};

use crate::wire::{self, Message};

/// `NetworkInitialization.CONFIG` and `LOGIN` are built with `networkProtocolVersion(0)`, and
/// `ChannelListManager.CHANNEL` keeps the default 0.
const FORGE_CHANNELS: [(&str, i32); 3] = [
    (wire::LOGIN_CHANNEL, 0),
    (wire::HANDSHAKE_CHANNEL, 0),
    (wire::CHANNEL_REGISTRATION, 0),
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum State {
    AwaitingModVersions,
    AwaitingChannelVersions,
    /// Waiting for the acknowledgement of `token`.
    AwaitingAck {
        token: i32,
    },
    Done,
}

/// Per-connection handshake, created for clients whose handshake address carries the Forge
/// marker.
#[derive(Debug)]
pub struct ForgeHandshake {
    state: State,
    /// Installed mods as (id, display name, version), for `ModVersions`.
    mods: Vec<(String, String, String)>,
    /// Registries still to send, in order.
    registries: Vec<SyncedRegistry>,
    next_registry: usize,
    config_files: Vec<(String, Vec<u8>)>,
    /// Forge's own channels and the mods' (name, network protocol version), for
    /// `ChannelVersions`.
    channels: Vec<(String, i32)>,
    client_mods: Vec<String>,
}

impl ForgeHandshake {
    /// `mod_channels` are the mods' Forge channels as (name, network protocol version).
    #[must_use]
    pub fn new(
        mods: Vec<(String, String, String)>,
        registries: Vec<SyncedRegistry>,
        config_files: Vec<(String, Vec<u8>)>,
        mod_channels: Vec<(String, i32)>,
    ) -> Self {
        let mut channels: Vec<(String, i32)> = FORGE_CHANNELS
            .iter()
            .map(|(name, version)| ((*name).to_string(), *version))
            .collect();
        channels.extend(mod_channels);
        Self {
            state: State::AwaitingModVersions,
            mods,
            registries,
            next_registry: 0,
            config_files,
            channels,
            client_mods: Vec::new(),
        }
    }

    /// Mod ids from the client's `ModVersions`.
    #[must_use]
    pub fn client_mods(&self) -> &[String] {
        &self.client_mods
    }

    /// Steps 1 and 2, right after login acknowledgement.
    #[must_use]
    pub fn start(&self) -> Vec<Outgoing> {
        let channels: Vec<&str> = [wire::HANDSHAKE_CHANNEL, wire::LOGIN_CHANNEL].to_vec();
        vec![
            Outgoing::Payload {
                channel: register::REGISTER_CHANNEL,
                data: register::encode(&channels),
            },
            Outgoing::Payload {
                channel: wire::HANDSHAKE_CHANNEL,
                data: wire::encode_mod_versions(&self.mods),
            },
        ]
    }

    const fn payload(data: Vec<u8>) -> Outgoing {
        Outgoing::Payload {
            channel: wire::HANDSHAKE_CHANNEL,
            data,
        }
    }

    /// The next `RegistryData`, or step 5 when every registry was sent.
    fn next_registry_or_finish(&mut self, token: i32) -> Step {
        if let Some(registry) = self.registries.get(self.next_registry) {
            self.next_registry += 1;
            let token = token + 1;
            self.state = State::AwaitingAck { token };
            return Step::Send(vec![Self::payload(wire::encode_registry_data(
                token,
                &registry.id,
                &registry.entries,
            ))]);
        }
        self.state = State::Done;
        Step::Done(
            self.config_files
                .iter()
                .map(|(name, contents)| Self::payload(wire::encode_config_data(name, contents)))
                .collect(),
        )
    }

    /// Feed a configuration-phase custom payload from the client.
    pub fn on_payload(&mut self, channel: &str, data: &[u8]) -> Step {
        if channel == register::REGISTER_CHANNEL && self.state != State::Done {
            return Step::Wait;
        }
        if channel != wire::HANDSHAKE_CHANNEL {
            return Step::NotHandled;
        }
        let Ok((message, body)) = wire::split(data) else {
            return Step::Disconnect(String::from("Malformed forge:handshake payload"));
        };
        match (self.state, message) {
            (State::AwaitingModVersions, Some(Message::ModVersions)) => {
                // `handleModVersions` only records the list on the server.
                self.client_mods = wire::decode_mod_versions(body).unwrap_or_default();
                self.state = State::AwaitingChannelVersions;
                let channels: Vec<(&str, i32)> = self
                    .channels
                    .iter()
                    .map(|(name, version)| (name.as_str(), *version))
                    .collect();
                Step::Send(vec![Self::payload(wire::encode_channel_versions(
                    &channels,
                ))])
            }
            (State::AwaitingChannelVersions, Some(Message::ChannelVersions)) => {
                // `NetworkRegistry.validateChannels`: a channel the client lacks is accepted (the
                // client's own check rejects it if it isn't optional) but not another version.
                let Ok(client) = wire::decode_channel_versions(body) else {
                    return Step::Disconnect(String::from("Malformed ChannelVersions"));
                };
                let mismatched: Vec<String> = self
                    .channels
                    .iter()
                    .filter_map(|(name, version)| {
                        client
                            .iter()
                            .find(|(c, _)| c == name)
                            .filter(|(_, v)| v != version)
                            .map(|(c, v)| format!("{c}: client {v}, server {version}"))
                    })
                    .collect();
                if !mismatched.is_empty() {
                    return Step::Disconnect(format!(
                        "Connection closed - mismatched mod channel list\n{}",
                        mismatched.join("\n")
                    ));
                }
                let token = 0;
                self.state = State::AwaitingAck { token };
                let names: Vec<&str> = self.registries.iter().map(|r| r.id.as_str()).collect();
                Step::Send(vec![Self::payload(wire::encode_registry_list(
                    token, &names,
                ))])
            }
            (State::AwaitingAck { token }, Some(Message::Acknowledge)) => {
                // `handleClientAck`: an unknown token closes the connection.
                match wire::decode_acknowledge(body) {
                    Ok(acked) if acked == token => self.next_registry_or_finish(token),
                    _ => Step::Disconnect(String::from("Illegal Acknowledge packet received")),
                }
            }
            (_, Some(Message::MismatchData)) => Step::Disconnect(String::from(
                "Connection closed - mismatched mod channel list",
            )),
            _ => Step::Wait,
        }
    }
}

#[cfg(test)]
#[allow(clippy::panic)]
mod tests {
    use super::*;

    fn client_message(message: Message, body: &[u8]) -> Vec<u8> {
        let mut out = vec![message as u8];
        out.extend_from_slice(body);
        out
    }

    #[test]
    fn runs_every_task_in_order() {
        let registry = SyncedRegistry {
            id: "minecraft:block".into(),
            optional: false,
            entries: vec![("minecraft:air".into(), 0)],
        };
        let config = vec![("forge-server.toml".to_string(), b"[server]\n".to_vec())];
        let mut handshake = ForgeHandshake::new(Vec::new(), vec![registry], config, Vec::new());
        assert_eq!(handshake.start().len(), 2);

        let mods = client_message(Message::ModVersions, &[0]);
        assert!(matches!(
            handshake.on_payload(wire::HANDSHAKE_CHANNEL, &mods),
            Step::Send(_)
        ));
        let channels = client_message(Message::ChannelVersions, &[0]);
        assert!(matches!(
            handshake.on_payload(wire::HANDSHAKE_CHANNEL, &channels),
            Step::Send(_)
        ));
        // Acknowledge the registry list (token 0), then the one registry (token 1).
        let ack0 = client_message(Message::Acknowledge, &[0]);
        assert!(matches!(
            handshake.on_payload(wire::HANDSHAKE_CHANNEL, &ack0),
            Step::Send(_)
        ));
        let ack1 = client_message(Message::Acknowledge, &[1]);
        let Step::Done(config) = handshake.on_payload(wire::HANDSHAKE_CHANNEL, &ack1) else {
            panic!("expected the end of the handshake");
        };
        assert_eq!(config.len(), 1);
    }

    #[test]
    fn a_wrong_token_disconnects() {
        let mut handshake = ForgeHandshake::new(Vec::new(), Vec::new(), Vec::new(), Vec::new());
        handshake.on_payload(
            wire::HANDSHAKE_CHANNEL,
            &client_message(Message::ModVersions, &[0]),
        );
        handshake.on_payload(
            wire::HANDSHAKE_CHANNEL,
            &client_message(Message::ChannelVersions, &[0]),
        );
        assert!(matches!(
            handshake.on_payload(
                wire::HANDSHAKE_CHANNEL,
                &client_message(Message::Acknowledge, &[5])
            ),
            Step::Disconnect(_)
        ));
    }
}
