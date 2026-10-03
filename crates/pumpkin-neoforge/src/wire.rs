//! Byte layouts of the NeoForge payloads Pumpkin speaks (NeoForge 26.3.x,
//! `net.neoforged.neoforge.network.payload`). Updating to a new NeoForge means re-reading the
//! classes cited on each item.

use pumpkin_fabric::wire::{ReadError, Reader, write_string, write_var_int};

/// `ModdedNetworkQueryPayload.ID`: the server's empty query, and the client's channel list.
pub const QUERY_CHANNEL: &str = "neoforge:register";
/// `ModdedNetworkPayload.ID`: the negotiated channels.
pub const NETWORK_CHANNEL: &str = "neoforge:network";
/// `ModdedNetworkSetupFailedPayload.ID`.
pub const SETUP_FAILED_CHANNEL: &str = "neoforge:modded_network_setup_failed";
/// `FrozenRegistrySyncStartPayload.TYPE`.
pub const SYNC_START_CHANNEL: &str = "neoforge:frozen_registry_sync_start";
/// `FrozenRegistryPayload.TYPE`.
pub const SYNC_CHANNEL: &str = "neoforge:frozen_registry";
/// `FrozenRegistrySyncCompletedPayload.TYPE`, sent by both sides.
pub const SYNC_COMPLETED_CHANNEL: &str = "neoforge:frozen_registry_sync_completed";

/// `ConfigFilePayload.TYPE`: a synced config file, in configuration and play.
pub const CONFIG_FILE_CHANNEL: &str = "neoforge:config_file";

/// NeoForge's own synced config, with every value at its default for a production server.
///
/// `NeoForgeSyncedConfig`, registered as `ModConfig.Type.SYNCED`, so `FancyModLoader` names it
/// `neoforge-synced.toml`. A NeoForge client reads these values (for example in
/// `Level.guardEntityTick`) and crashes if the server never sends the file.
pub const NEOFORGE_SYNCED_CONFIG: (&str, &str) = (
    "neoforge-synced.toml",
    "removeErroringBlockEntities = false\n\
     removeErroringEntities = false\n\
     fullBoundingBoxLadders = false\n\
     permissionHandler = \"neoforge:default_handler\"\n\
     advertiseDedicatedServerToLan = false\n",
);

/// `NetworkRegistry.BUILTIN_PAYLOADS`: channels both sides listen on before negotiation
/// (`getInitialListeningChannels`).
pub const BUILTIN_CHANNELS: [&str; 7] = [
    "minecraft:register",
    "minecraft:unregister",
    QUERY_CHANNEL,
    NETWORK_CHANNEL,
    SETUP_FAILED_CHANNEL,
    "c:version",
    "c:register",
];

/// Most channels a client may list per protocol, and protocols per query.
const MAX_CHANNELS: usize = 4096;
const MAX_PROTOCOLS: usize = 8;

/// `ConnectionProtocol`, by ordinal (`HANDSHAKING`, `PLAY`, `STATUS`, `LOGIN`, `CONFIGURATION`).
/// Only the two that carry custom payloads after login are negotiated.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Protocol {
    Play = 1,
    Configuration = 4,
}

impl Protocol {
    const fn from_ordinal(ordinal: i32) -> Option<Self> {
        match ordinal {
            1 => Some(Self::Play),
            4 => Some(Self::Configuration),
            _ => None,
        }
    }
}

/// `PacketFlow`, by ordinal.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Flow {
    Serverbound = 0,
    Clientbound = 1,
}

/// `ModdedNetworkQueryComponent`: one channel a side registered.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Component {
    pub id: String,
    pub version: String,
    /// None for channels both sides send on.
    pub flow: Option<Flow>,
    pub optional: bool,
}

/// The server's `ModdedNetworkQueryPayload` (`NetworkRegistry` sends it with an empty map).
#[must_use]
pub fn encode_empty_query() -> Vec<u8> {
    vec![0]
}

/// The client's `ModdedNetworkQueryPayload`: its channels per protocol.
///
/// A `ByteBufCodecs.map` of protocol ordinal to a collection of `ModdedNetworkQueryComponent`
/// (id, version, optional flow, optional flag). Protocols other than play and configuration
/// are skipped.
pub fn decode_query(payload: &[u8]) -> Result<Vec<(Protocol, Vec<Component>)>, ReadError> {
    let mut reader = Reader::new(payload);
    let protocols = reader.length(MAX_PROTOCOLS)?;
    let mut out = Vec::with_capacity(protocols);
    for _ in 0..protocols {
        let protocol = Protocol::from_ordinal(reader.var_int()?);
        let count = reader.length(MAX_CHANNELS)?;
        let mut components = Vec::with_capacity(count);
        for _ in 0..count {
            let id = reader.string()?.to_string();
            let version = reader.string()?.to_string();
            let flow = if reader.byte()? == 0 {
                None
            } else {
                match reader.var_int()? {
                    0 => Some(Flow::Serverbound),
                    1 => Some(Flow::Clientbound),
                    other => return Err(ReadError::BadLength(other)),
                }
            };
            let optional = reader.byte()? != 0;
            components.push(Component {
                id,
                version,
                flow,
                optional,
            });
        }
        if let Some(protocol) = protocol {
            out.push((protocol, components));
        }
    }
    Ok(out)
}

/// `ModdedNetworkPayload` / `NetworkPayloadSetup`: per protocol, each negotiated channel as
/// `NetworkChannel` (id, chosen version), keyed by id.
#[must_use]
pub fn encode_setup(setup: &[(Protocol, Vec<(String, String)>)]) -> Vec<u8> {
    let mut out = Vec::new();
    write_var_int(&mut out, setup.len() as i32);
    for (protocol, channels) in setup {
        write_var_int(&mut out, *protocol as i32);
        write_var_int(&mut out, channels.len() as i32);
        for (id, version) in channels {
            write_string(&mut out, id);
            write_string(&mut out, id);
            write_string(&mut out, version);
        }
    }
    out
}

/// `ConfigFilePayload`: the file name, then its contents as a byte array.
#[must_use]
pub fn encode_config_file(name: &str, contents: &[u8]) -> Vec<u8> {
    let mut out = Vec::new();
    write_string(&mut out, name);
    write_var_int(&mut out, contents.len() as i32);
    out.extend_from_slice(contents);
    out
}

/// `FrozenRegistrySyncStartPayload`: the registries the client must receive before the
/// completion payload.
#[must_use]
pub fn encode_sync_start(registries: &[&str]) -> Vec<u8> {
    let mut out = Vec::new();
    write_var_int(&mut out, registries.len() as i32);
    for registry in registries {
        write_string(&mut out, registry);
    }
    out
}

/// `FrozenRegistryPayload`: the registry name, then its `RegistrySnapshot` (raw id to name,
/// sorted by id, then the aliases, none here).
#[must_use]
pub fn encode_registry(registry: &str, entries: &[(String, u32)]) -> Vec<u8> {
    let mut sorted: Vec<&(String, u32)> = entries.iter().collect();
    sorted.sort_by_key(|(_, raw)| *raw);
    let mut out = Vec::new();
    write_string(&mut out, registry);
    write_var_int(&mut out, sorted.len() as i32);
    for (name, raw) in sorted {
        write_var_int(&mut out, *raw as i32);
        write_string(&mut out, name);
    }
    write_var_int(&mut out, 0);
    out
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn decodes_a_client_query() {
        // One configuration channel: "a:b", version "1", clientbound, optional.
        let mut payload = vec![1, 4, 1];
        write_string(&mut payload, "a:b");
        write_string(&mut payload, "1");
        payload.extend([1, 1, 1]);
        let query = decode_query(&payload).unwrap();
        assert_eq!(
            query,
            vec![(
                Protocol::Configuration,
                vec![Component {
                    id: "a:b".into(),
                    version: "1".into(),
                    flow: Some(Flow::Clientbound),
                    optional: true,
                }]
            )]
        );
    }

    #[test]
    fn rejects_a_truncated_query() {
        assert!(decode_query(&[1, 4, 1]).is_err());
    }
}
