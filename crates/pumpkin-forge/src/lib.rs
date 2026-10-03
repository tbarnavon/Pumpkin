//! MinecraftForge client support: detection and the `forge:handshake` configuration tasks.
//!
//! Loader-specific code only, like `pumpkin-fabric` and `pumpkin-neoforge`, sharing
//! `pumpkin-fabric`'s byte helpers, `minecraft:register` and registry list. The byte layouts are
//! in [`wire`]; [`handshake`] sequences them the way Forge's server does.

pub mod handshake;
pub mod wire;
