//! Fabric client support: channel registration and registry sync during the configuration phase.
//!
//! Loader-specific code only. The registry entries themselves come from `pumpkin_data::dynamic`,
//! filled by `pumpkin-registry-ext`. The byte layouts are in [`wire`]; [`handshake`] sequences them
//! the way Fabric's server does.

pub mod handshake;
pub mod sync_map;
pub mod wire;
