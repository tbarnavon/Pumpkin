use serde::{Deserialize, Serialize};

/// Content from Fabric mods, served to modded clients (see `docs/DESIGN.md`).
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
    /// Keep the Bedrock listener on while modded content is enabled. Bedrock clients cannot see
    /// modded blocks or items, so it is turned off unless this is set.
    pub force_bedrock: bool,
}
