//! The "common" channel negotiation shared by Fabric and `NeoForge` (fabric-api PR #3244):
//! `c:version` and `c:register`.

use super::{ReadError, Reader, write_string, write_var_int};

/// `CommonVersionPayload.TYPE` (`impl/networking/CommonVersionPayload.java:26`).
pub const VERSION_CHANNEL: &str = "c:version";
/// `CommonRegisterPayload.TYPE` (`impl/networking/CommonRegisterPayload.java:31`).
pub const REGISTER_CHANNEL: &str = "c:register";

/// `CommonPacketsImpl.SUPPORTED_COMMON_PACKET_VERSIONS` (`impl/networking/CommonPacketsImpl.java:35`).
pub const SUPPORTED_VERSIONS: &[i32] = &[1];

/// `CommonRegisterPayload.PLAY_PROTOCOL` / `CONFIGURATION_PROTOCOL`.
pub const PLAY_PROTOCOL: &str = "play";
pub const CONFIGURATION_PROTOCOL: &str = "configuration";

/// Upper bound on list lengths read from clients.
const MAX_ENTRIES: usize = 4096;

/// `c:version` body: `FriendlyByteBuf.writeVarIntArray` (`CommonVersionPayload.java:32`).
#[must_use]
pub fn encode_version(versions: &[i32]) -> Vec<u8> {
    let mut out = Vec::new();
    write_var_int(&mut out, versions.len() as i32);
    for version in versions {
        write_var_int(&mut out, *version);
    }
    out
}

pub fn decode_version(payload: &[u8]) -> Result<Vec<i32>, ReadError> {
    let mut reader = Reader::new(payload);
    let count = reader.length(MAX_ENTRIES)?;
    (0..count).map(|_| reader.var_int()).collect()
}

/// `CommonPacketsImpl.getHighestCommonVersion` (`CommonPacketsImpl.java:126`): the highest
/// version both sides support.
#[must_use]
pub fn negotiate(client: &[i32]) -> Option<i32> {
    SUPPORTED_VERSIONS
        .iter()
        .filter(|v| client.contains(v))
        .max()
        .copied()
}

/// A `c:register` body.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommonRegister {
    pub version: i32,
    pub protocol: String,
    pub channels: Vec<String>,
}

/// `CommonRegisterPayload.write` (`CommonRegisterPayload.java:46-50`): `VarInt` version,
/// `String` protocol, then `Identifier.STREAM_CODEC` collection (`VarInt` count, strings).
#[must_use]
pub fn encode_register(version: i32, protocol: &str, channels: &[&str]) -> Vec<u8> {
    let mut out = Vec::new();
    write_var_int(&mut out, version);
    write_string(&mut out, protocol);
    write_var_int(&mut out, channels.len() as i32);
    for channel in channels {
        write_string(&mut out, channel);
    }
    out
}

pub fn decode_register(payload: &[u8]) -> Result<CommonRegister, ReadError> {
    let mut reader = Reader::new(payload);
    let version = reader.var_int()?;
    let protocol = reader.string()?.to_string();
    let count = reader.length(MAX_ENTRIES)?;
    let channels = (0..count)
        .map(|_| reader.string().map(str::to_string))
        .collect::<Result<_, _>>()?;
    Ok(CommonRegister {
        version,
        protocol,
        channels,
    })
}
