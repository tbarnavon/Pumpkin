use serde::{Deserialize, Serialize};

/// Content from mods, served to modded clients (see `docs/DESIGN.md`).
///
/// Off by default, so a vanilla server never reads mod data or lets plugins use the `modded` API.
#[derive(Deserialize, Serialize, Default)]
#[serde(default)]
pub struct ModdedConfig {
    /// Whether to load mod data from the `mod-data` folder and let plugins use the `modded` API
    /// (block and item hooks, modded menus).
    pub enabled: bool,
    /// Whether to run Fabric's login handshake (`fabric:registry/sync`) with clients when mod
    /// data is loaded. Fabric clients with the mods installed need it to join. Other loaders'
    /// handshakes get their own switch.
    pub fabric_handshake: bool,
    /// Whether to run NeoForge's channel negotiation and registry sync with NeoForge clients.
    /// Unlike Fabric's, it also runs without mod data, so plain NeoForge clients join as
    /// NeoForge connections.
    pub neoforge_handshake: bool,
    /// Whether to run Forge's `forge:handshake` tasks with clients whose handshake carries
    /// Forge's marker. Also runs without mod data.
    pub forge_handshake: bool,
    /// Keep the Bedrock listener on while modded content is enabled. Bedrock clients cannot see
    /// modded blocks or items, so it is turned off unless this is set.
    pub force_bedrock: bool,
}
