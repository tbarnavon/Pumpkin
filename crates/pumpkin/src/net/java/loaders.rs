//! Which mod loader a client runs, found during the configuration phase, and its handshake.
//!
//! A Forge client says so in the handshake's server address, before login. For the others the
//! server announces Fabric's and NeoForge's channels and pings: a NeoForge client answers the
//! `neoforge:register` query, a Fabric client only `minecraft:register`, a vanilla client
//! neither; all of them answer before the pong, so the pong decides.

use pumpkin_fabric::handshake::{FabricHandshake, Outgoing, PING_ID, Step};
use pumpkin_fabric::wire::{register, registry_sync};
use pumpkin_forge::handshake::ForgeHandshake;
use pumpkin_neoforge::handshake::NeoForgeHandshake;

/// What the configuration phase does with each loader, from the `[modded]` config.
pub struct LoaderOptions {
    /// Namespaces of the installed mods; empty without mod data.
    pub mod_namespaces: Vec<String>,
    pub fabric: bool,
    pub neoforge: bool,
    pub forge: bool,
    /// Whether the handshake's server address carried Forge's marker.
    pub forge_client: bool,
}

pub enum LoaderHandshake {
    Detecting {
        options: LoaderOptions,
        /// The client's `minecraft:register` payloads, kept for the Fabric handshake.
        registers: Vec<Vec<u8>>,
        neoforge_query: Option<Vec<u8>>,
    },
    Fabric(FabricHandshake),
    NeoForge(NeoForgeHandshake),
    Forge(ForgeHandshake),
    /// A vanilla client on a server without mods, or a loader whose handshake is off.
    Vanilla,
}

impl LoaderHandshake {
    /// The handshake to run after login, or none when neither loader's handshake can apply.
    #[must_use]
    pub fn new(options: LoaderOptions) -> Option<Self> {
        if options.forge_client && options.forge {
            let mods = options
                .mod_namespaces
                .iter()
                .map(|id| (id.clone(), id.clone(), String::from("0")))
                .collect();
            let (name, contents) = pumpkin_forge::wire::FORGE_SERVER_CONFIG;
            return Some(Self::Forge(ForgeHandshake::new(
                mods,
                pumpkin_fabric::sync_map::build(),
                vec![(name.to_string(), contents.as_bytes().to_vec())],
            )));
        }
        let fabric = options.fabric && !options.mod_namespaces.is_empty();
        (fabric || options.neoforge).then_some(Self::Detecting {
            options,
            registers: Vec::new(),
            neoforge_query: None,
        })
    }

    /// Packets to send right after login acknowledgement, before anything vanilla.
    #[must_use]
    pub fn start(&self) -> Vec<Outgoing> {
        if let Self::Forge(handshake) = self {
            return handshake.start();
        }
        let mut channels: Vec<&str> = pumpkin_fabric::handshake::SERVER_CONFIG_CHANNELS.to_vec();
        channels.extend(pumpkin_neoforge::wire::BUILTIN_CHANNELS);
        channels.sort_unstable();
        channels.dedup();
        vec![
            Outgoing::Payload {
                channel: register::REGISTER_CHANNEL,
                data: register::encode(&channels),
            },
            Outgoing::Payload {
                channel: pumpkin_neoforge::wire::QUERY_CHANNEL,
                data: pumpkin_neoforge::wire::encode_empty_query(),
            },
            Outgoing::Ping(PING_ID),
        ]
    }

    /// Feed a configuration-phase custom payload from the client.
    pub fn on_payload(&mut self, channel: &str, data: &[u8]) -> Step {
        match self {
            Self::Detecting {
                registers,
                neoforge_query,
                ..
            } => match channel {
                register::REGISTER_CHANNEL => {
                    registers.push(data.to_vec());
                    Step::Wait
                }
                pumpkin_neoforge::wire::QUERY_CHANNEL => {
                    *neoforge_query = Some(data.to_vec());
                    Step::Wait
                }
                _ => Step::NotHandled,
            },
            Self::Fabric(handshake) => handshake.on_payload(channel, data),
            Self::NeoForge(handshake) => handshake.on_payload(channel, data),
            Self::Forge(handshake) => handshake.on_payload(channel, data),
            Self::Vanilla => Step::NotHandled,
        }
    }

    /// Feed a configuration-phase pong: the end of detection.
    pub fn on_pong(&mut self, id: i32) -> Step {
        let Self::Detecting {
            options,
            registers,
            neoforge_query,
        } = self
        else {
            return Step::NotHandled;
        };
        if id != PING_ID {
            return Step::NotHandled;
        }
        let options = std::mem::replace(
            options,
            LoaderOptions {
                mod_namespaces: Vec::new(),
                fabric: false,
                neoforge: false,
                forge: false,
                forge_client: false,
            },
        );
        let registers = std::mem::take(registers);
        let neoforge_query = neoforge_query.take();
        let neoforge_query_seen = neoforge_query.is_some();
        let mods = !options.mod_namespaces.is_empty();

        if let Some(query) = neoforge_query
            && options.neoforge
        {
            let listens_on: Vec<String> =
                registers.iter().flat_map(|r| register::decode(r)).collect();
            let mut handshake = NeoForgeHandshake::new(
                pumpkin_neoforge::handshake::server_channels(),
                pumpkin_fabric::sync_map::build(),
                {
                    let (name, contents) = pumpkin_neoforge::wire::NEOFORGE_SYNCED_CONFIG;
                    vec![(name.to_string(), contents.as_bytes().to_vec())]
                },
            );
            let step = handshake.on_detected(&query, &listens_on);
            tracing::debug!("NeoForge client");
            *self = Self::NeoForge(handshake);
            return step;
        }
        let fabric_client = registers.iter().any(|r| {
            register::decode(r)
                .iter()
                .any(|c| c == registry_sync::SYNC_CHANNEL)
        });
        if fabric_client && options.fabric && mods {
            let mut handshake = FabricHandshake::new(options.mod_namespaces, Vec::new());
            // `minecraft:register` lists channels separated by NUL bytes.
            let joined = registers.join(&0u8);
            let step = handshake.on_payload(register::REGISTER_CHANNEL, &joined);
            tracing::debug!("Fabric client");
            *self = Self::Fabric(handshake);
            return step;
        }
        tracing::debug!(
            neoforge_query = neoforge_query_seen,
            registers = registers.len(),
            "Vanilla client, or a loader whose handshake is off"
        );
        if mods {
            return Step::Disconnect(format!(
                "This server requires these mods on your client: {}\n\
                 Install them with Fabric Loader and Fabric API.",
                options.mod_namespaces.join(", ")
            ));
        }
        *self = Self::Vanilla;
        Step::Done(Vec::new())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn detecting(mods: &[&str], neoforge: bool) -> LoaderHandshake {
        LoaderHandshake::new(LoaderOptions {
            mod_namespaces: mods.iter().map(|m| (*m).to_string()).collect(),
            fabric: true,
            neoforge,
            forge: true,
            forge_client: false,
        })
        .unwrap()
    }

    #[test]
    fn a_neoforge_query_before_the_pong_means_neoforge() {
        let mut handshake = detecting(&[], true);
        handshake.on_payload(register::REGISTER_CHANNEL, b"c:version\0c:register");
        handshake.on_payload(pumpkin_neoforge::wire::QUERY_CHANNEL, &[0]);
        assert!(matches!(handshake.on_pong(PING_ID), Step::Send(_)));
        assert!(matches!(handshake, LoaderHandshake::NeoForge(_)));
    }

    #[test]
    fn a_register_with_fabric_registry_sync_means_fabric() {
        let mut handshake = detecting(&["mod"], true);
        handshake.on_payload(register::REGISTER_CHANNEL, b"fabric:registry/sync");
        assert!(matches!(handshake.on_pong(PING_ID), Step::Send(_)));
        assert!(matches!(handshake, LoaderHandshake::Fabric(_)));
    }

    #[test]
    fn a_vanilla_client_joins_only_without_mods() {
        assert!(matches!(
            detecting(&[], true).on_pong(PING_ID),
            Step::Done(_)
        ));
        assert!(matches!(
            detecting(&["mod"], true).on_pong(PING_ID),
            Step::Disconnect(_)
        ));
    }

    #[test]
    fn the_forge_marker_skips_detection() {
        let handshake = LoaderHandshake::new(LoaderOptions {
            mod_namespaces: Vec::new(),
            fabric: true,
            neoforge: true,
            forge: true,
            forge_client: true,
        });
        assert!(matches!(handshake, Some(LoaderHandshake::Forge(_))));
    }

    #[test]
    fn no_handshake_without_mods_unless_neoforge_is_on() {
        assert!(
            LoaderHandshake::new(LoaderOptions {
                mod_namespaces: Vec::new(),
                fabric: true,
                neoforge: false,
                forge: true,
                forge_client: false,
            })
            .is_none()
        );
    }
}
