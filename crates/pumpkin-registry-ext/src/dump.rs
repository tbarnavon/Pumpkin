//! The JSON written by the Extractor's mod dump (`ModDump.kt`), one file per mod.

use std::collections::BTreeMap;

use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct ModDump {
    pub namespace: String,
    pub blocks: Vec<BlockDump>,
    pub items: Vec<ItemDump>,
    pub block_entity_types: Vec<NamedEntry>,
    pub menus: Vec<NamedEntry>,
    pub data_component_types: Vec<NamedEntry>,
    pub entity_types: Vec<NamedEntry>,
    pub tags: TagsDump,
    /// Recipe id to recipe JSON, as encoded by `Recipe.DIRECT_CODEC`.
    pub recipes: BTreeMap<String, serde_json::Value>,
    /// Loot table id to loot table JSON, as encoded by `LootTable.DIRECT_CODEC`.
    pub loot_tables: BTreeMap<String, serde_json::Value>,
}

/// An entry of a registry where only the name and raw id matter.
#[derive(Debug, Deserialize)]
pub struct NamedEntry {
    pub name: String,
    pub raw_id: u16,
}

#[derive(Debug, Deserialize)]
pub struct BlockDump {
    pub name: String,
    pub raw_id: u16,
    pub hardness: f32,
    pub blast_resistance: f32,
    pub map_color: u8,
    pub slipperiness: f32,
    pub velocity_multiplier: f32,
    pub jump_velocity_multiplier: f32,
    /// Namespaced item name; `minecraft:air` when the block has no item.
    pub item: String,
    pub flammable: Option<FlammableDump>,
    pub experience: Option<ExperienceDump>,
    pub properties: Vec<PropertyDump>,
    /// Index into `states`.
    pub default_state: usize,
    pub states: Vec<StateDump>,
}

#[derive(Debug, Deserialize)]
pub struct FlammableDump {
    pub spread_chance: u8,
    pub burn_chance: u8,
}

#[derive(Debug, Deserialize)]
pub struct ExperienceDump {
    pub min_inclusive: i32,
    pub max_inclusive: i32,
}

#[derive(Debug, Deserialize)]
pub struct PropertyDump {
    pub name: String,
    pub values: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct StateDump {
    pub raw_id: u16,
    pub values: Vec<String>,
    pub state_flags: u16,
    pub side_flags: u8,
    pub instrument: String,
    pub luminance: u8,
    pub piston_behavior: String,
    pub hardness: f32,
    pub opacity: u8,
    pub random_ticks: bool,
    pub collision_shapes: Vec<ShapeDump>,
    pub outline_shapes: Vec<ShapeDump>,
    pub block_entity_type: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ShapeDump {
    pub min: [f64; 3],
    pub max: [f64; 3],
}

#[derive(Debug, Deserialize)]
pub struct ItemDump {
    pub name: String,
    pub raw_id: u16,
    /// Default components as base64 NBT (named root compound, empty name).
    pub components_nbt: String,
}

/// Resolved tag contents: tag name to every entry name, after all datapacks were applied.
#[derive(Debug, Deserialize)]
pub struct TagsDump {
    pub block: BTreeMap<String, Vec<String>>,
    pub item: BTreeMap<String, Vec<String>>,
}
