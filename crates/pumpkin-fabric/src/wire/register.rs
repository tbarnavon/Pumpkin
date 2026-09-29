//! `minecraft:register` / `minecraft:unregister`: the legacy channel lists.
//!
//! fabric-networking-api-v1 `impl/networking/RegistrationPayload.java:42-73`: channel identifiers
//! as US-ASCII, separated by a single `\0`, no length prefix and no trailing separator. Invalid
//! identifiers are skipped with a warning on the Java side; here they are skipped silently.

/// `NetworkingImpl.REGISTER_CHANNEL` (`impl/networking/NetworkingImpl.java:37`).
pub const REGISTER_CHANNEL: &str = "minecraft:register";

#[must_use]
pub fn encode(channels: &[&str]) -> Vec<u8> {
    let mut out = Vec::new();
    for (index, channel) in channels.iter().enumerate() {
        if index > 0 {
            out.push(0);
        }
        out.extend_from_slice(channel.as_bytes());
    }
    out
}

/// Channel names in the payload. Empty and non-ASCII names are dropped.
#[must_use]
pub fn decode(payload: &[u8]) -> Vec<String> {
    payload
        .split(|b| *b == 0)
        .filter(|name| !name.is_empty() && name.is_ascii())
        .filter_map(|name| std::str::from_utf8(name).ok())
        .filter(|name| name.contains(':'))
        .map(str::to_string)
        .collect()
}
