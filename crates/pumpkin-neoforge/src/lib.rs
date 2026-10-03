//! NeoForge client support: channel negotiation and registry sync during the configuration phase.
//!
//! Loader-specific code only, like `pumpkin-fabric`, whose `minecraft:register`, `c:` channel and
//! registry list code it shares. The byte layouts are in [`wire`], the channel negotiation in
//! [`negotiation`], and [`handshake`] sequences them the way NeoForge's server does.

pub mod handshake;
pub mod negotiation;
pub mod wire;
