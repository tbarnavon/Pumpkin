//! Byte layouts of the Forge payloads Pumpkin speaks (MinecraftForge for Minecraft 26.3,
//! `net.minecraftforge.network`). Updating to a new Forge means re-reading the classes cited on
//! each item.

use pumpkin_fabric::wire::{ReadError, Reader, write_string, write_var_int};

/// `NetworkInitialization.HANDSHAKE_NAME`: the `SimpleChannel` all handshake messages use.
pub const HANDSHAKE_CHANNEL: &str = "forge:handshake";
/// `NetworkInitialization.LOGIN_NAME`.
pub const LOGIN_CHANNEL: &str = "forge:login";
/// `ChannelListManager.NAME`.
pub const CHANNEL_REGISTRATION: &str = "forge:channel_registration";

/// `NetworkContext.MARKER`: a Forge client appends `\0FORGE`, optionally followed by its network
/// version, to the server address in the handshake packet.
const MARKER: &str = "FORGE";

/// The Forge network version from the handshake's server address (`processIntention`), or none
/// for a client that isn't Forge.
#[must_use]
pub fn forge_marker(server_address: &str) -> Option<i32> {
    if !server_address.contains('\0') {
        return None;
    }
    server_address
        .split('\0')
        .find_map(|part| part.strip_prefix(MARKER))
        .map(|version| version.parse().unwrap_or(0))
}

/// `SimpleChannel` message ids on `forge:handshake`, in `NetworkInitialization.CONFIG`'s
/// registration order (`nextIndex` counts up from 0).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Message {
    Acknowledge = 0,
    ModVersions = 1,
    ChannelVersions = 2,
    RegistryList = 3,
    RegistryData = 4,
    ConfigData = 5,
    MismatchData = 6,
}

impl Message {
    const fn from_id(id: i32) -> Option<Self> {
        Some(match id {
            0 => Self::Acknowledge,
            1 => Self::ModVersions,
            2 => Self::ChannelVersions,
            3 => Self::RegistryList,
            4 => Self::RegistryData,
            5 => Self::ConfigData,
            6 => Self::MismatchData,
            _ => return None,
        })
    }
}

/// Most entries a client may send in one list.
const MAX_ENTRIES: usize = 4096;
/// `ModVersions.STRING_CODEC`: `ByteBufCodecs.stringUtf8(0x100)`.
const MAX_MOD_STRING: usize = 0x100;

fn message(message: Message) -> Vec<u8> {
    let mut out = Vec::new();
    write_var_int(&mut out, message as i32);
    out
}

/// The message id of a `forge:handshake` payload, and its body.
pub fn split(payload: &[u8]) -> Result<(Option<Message>, &[u8]), ReadError> {
    let mut reader = Reader::new(payload);
    let id = reader.var_int()?;
    Ok((Message::from_id(id), reader.rest()))
}

/// `ModVersions`: mod id to (display name, version).
#[must_use]
pub fn encode_mod_versions(mods: &[(String, String, String)]) -> Vec<u8> {
    let mut out = message(Message::ModVersions);
    write_var_int(&mut out, mods.len() as i32);
    for (id, name, version) in mods {
        write_string(&mut out, id);
        write_string(&mut out, name);
        write_string(&mut out, version);
    }
    out
}

/// The client's `ModVersions`: its mod ids.
pub fn decode_mod_versions(body: &[u8]) -> Result<Vec<String>, ReadError> {
    let mut reader = Reader::new(body);
    let count = reader.length(MAX_ENTRIES)?;
    let mut ids = Vec::with_capacity(count);
    for _ in 0..count {
        let id = bounded(&mut reader)?;
        bounded(&mut reader)?;
        bounded(&mut reader)?;
        ids.push(id);
    }
    Ok(ids)
}

fn bounded(reader: &mut Reader<'_>) -> Result<String, ReadError> {
    let value = reader.string()?;
    if value.chars().count() > MAX_MOD_STRING {
        return Err(ReadError::BadLength(value.len() as i32));
    }
    Ok(value.to_string())
}

/// `ChannelVersions`: channel to network protocol version.
#[must_use]
pub fn encode_channel_versions(channels: &[(&str, i32)]) -> Vec<u8> {
    let mut out = message(Message::ChannelVersions);
    write_var_int(&mut out, channels.len() as i32);
    for (channel, version) in channels {
        write_string(&mut out, channel);
        write_var_int(&mut out, *version);
    }
    out
}

/// The client's `ChannelVersions`.
pub fn decode_channel_versions(body: &[u8]) -> Result<Vec<(String, i32)>, ReadError> {
    let mut reader = Reader::new(body);
    let count = reader.length(MAX_ENTRIES)?;
    let mut channels = Vec::with_capacity(count);
    for _ in 0..count {
        let channel = reader.string()?.to_string();
        channels.push((channel, reader.var_int()?));
    }
    Ok(channels)
}

/// `RegistryList`: the acknowledgement token, the registries to sync, and the synced custom
/// datapack registries (none here).
#[must_use]
pub fn encode_registry_list(token: i32, registries: &[&str]) -> Vec<u8> {
    let mut out = message(Message::RegistryList);
    write_var_int(&mut out, token);
    write_var_int(&mut out, registries.len() as i32);
    for registry in registries {
        write_string(&mut out, registry);
    }
    write_var_int(&mut out, 0);
    out
}

/// `RegistryData`: the token, the registry name, then its `ForgeRegistry.Snapshot` (name to raw
/// id, then aliases, overrides and blocked ids, all empty here).
#[must_use]
pub fn encode_registry_data(token: i32, registry: &str, entries: &[(String, u32)]) -> Vec<u8> {
    let mut out = message(Message::RegistryData);
    write_var_int(&mut out, token);
    write_string(&mut out, registry);
    write_var_int(&mut out, entries.len() as i32);
    for (name, raw) in entries {
        write_string(&mut out, name);
        write_var_int(&mut out, *raw as i32);
    }
    write_var_int(&mut out, 0);
    write_var_int(&mut out, 0);
    write_var_int(&mut out, 0);
    out
}

/// The client's `Acknowledge`: its token.
pub fn decode_acknowledge(body: &[u8]) -> Result<i32, ReadError> {
    Reader::new(body).var_int()
}

/// `ConfigData`: the file name, then its contents as a byte array.
#[must_use]
pub fn encode_config_data(name: &str, contents: &[u8]) -> Vec<u8> {
    let mut out = message(Message::ConfigData);
    write_string(&mut out, name);
    write_var_int(&mut out, contents.len() as i32);
    out.extend_from_slice(contents);
    out
}

/// Forge's own server config, with its defaults.
///
/// `ForgeConfig.Server`, registered as `ModConfig.Type.SERVER`, so `forge-server.toml`. A Forge
/// client on a modded server loads its server configs only from the server
/// (`ForgeHooks.handleClientConfigurationComplete`).
pub const FORGE_SERVER_CONFIG: (&str, &str) = (
    "forge-server.toml",
    "[server]\n\
     removeErroringBlockEntities = false\n\
     removeErroringEntities = false\n\
     fullBoundingBoxLadders = false\n\
     permissionHandler = \"forge:default_handler\"\n\
     advertiseDedicatedServerToLan = true\n",
);

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn reads_the_forge_marker() {
        assert_eq!(forge_marker("localhost"), None);
        assert_eq!(forge_marker("localhost\0FORGE"), Some(0));
        assert_eq!(forge_marker("localhost\0FORGE3"), Some(3));
        // A BungeeCord-style address with other parts.
        assert_eq!(forge_marker("host\0address\0FORGE"), Some(0));
    }

    #[test]
    fn splits_a_message() {
        let payload = encode_channel_versions(&[("forge:handshake", 0)]);
        let (message, body) = split(&payload).unwrap();
        assert_eq!(message, Some(Message::ChannelVersions));
        assert_eq!(
            decode_channel_versions(body).unwrap(),
            vec![("forge:handshake".to_string(), 0)]
        );
    }
}
