//! Modded blocks and block states.

use std::collections::{HashMap, HashSet};
use std::sync::OnceLock;

use pumpkin_util::math::boundingbox::BoundingBox;
use pumpkin_util::math::experience::Experience;

use super::names::{NameTables, SyncedRegistry};
use super::{FreezeError, leak_str, validate_modded_name};
use crate::block_properties::{
    BLOCK_ENTITY_TYPES, BlockProperties, COLLISION_SHAPES, NoteblockInstrument,
};
use crate::block_state::PistonBehavior;
use crate::blocks::Flammable;
use crate::{Block, BlockId, BlockState, BlockStateId};

const REGISTRY: &str = "minecraft:block";

/// A modded block as dumped from the running mod.
#[derive(Debug, Clone)]
pub struct BlockDef {
    /// Namespaced name, for example `storagedrawers:oak_full_drawers_1`.
    pub name: String,
    pub hardness: f32,
    pub blast_resistance: f32,
    pub map_color: u8,
    pub slipperiness: f32,
    pub velocity_multiplier: f32,
    pub jump_velocity_multiplier: f32,
    /// Raw id of the block's item form, or 0 (air) if it has none.
    pub item_id: u16,
    pub flammable: Option<Flammable>,
    pub experience: Option<Experience>,
    /// Property names with their allowed values.
    pub properties: Vec<(String, Vec<String>)>,
    /// States in the mod's `StateDefinition.getPossibleStates()` order. Fabric's client assigns
    /// state ids in exactly this order.
    pub states: Vec<StateDef>,
    /// Index into `states` of the default state.
    pub default_state: usize,
}

/// One state of a [`BlockDef`].
#[derive(Debug, Clone)]
pub struct StateDef {
    /// Value of each property, in the order of [`BlockDef::properties`].
    pub values: Vec<String>,
    pub state_flags: u16,
    pub side_flags: u8,
    pub instrument: NoteblockInstrument,
    pub luminance: u8,
    pub piston_behavior: PistonBehavior,
    pub hardness: f32,
    pub opacity: u8,
    pub collision_shapes: Vec<BoundingBox>,
    pub outline_shapes: Vec<BoundingBox>,
    /// Block-entity type name (vanilla path or namespaced modded name), if the state has one.
    pub block_entity_type: Option<String>,
    pub random_ticks: bool,
}

pub(crate) struct BlockTables {
    blocks: Box<[&'static Block]>,
    by_name: HashMap<&'static str, &'static Block>,
    /// Modded item id to the first modded block that has it as its item.
    by_item: HashMap<u16, &'static Block>,
    /// Indexed by `state id - BlockStateId::VANILLA_COUNT`.
    states: Box<[ModdedState]>,
    /// Shapes past the end of [`COLLISION_SHAPES`].
    shapes: Box<[BoundingBox]>,
}

struct ModdedState {
    block: BlockId,
    state: &'static BlockState,
    props: &'static [(&'static str, &'static str)],
    random_ticks: bool,
}

static BLOCKS: OnceLock<BlockTables> = OnceLock::new();

fn invalid(name: &str, reason: impl Into<String>) -> FreezeError {
    FreezeError::InvalidBlock {
        name: name.to_string(),
        reason: reason.into(),
    }
}

/// Deduplicates shapes against the vanilla table and each other.
struct ShapeInterner {
    index: HashMap<[u64; 6], u16>,
    extra: Vec<BoundingBox>,
}

impl ShapeInterner {
    fn new() -> Self {
        let mut index = HashMap::new();
        for (i, shape) in COLLISION_SHAPES.iter().enumerate() {
            index.entry(Self::key(shape)).or_insert(i as u16);
        }
        Self {
            index,
            extra: Vec::new(),
        }
    }

    fn key(shape: &BoundingBox) -> [u64; 6] {
        [
            shape.min.x.to_bits(),
            shape.min.y.to_bits(),
            shape.min.z.to_bits(),
            shape.max.x.to_bits(),
            shape.max.y.to_bits(),
            shape.max.z.to_bits(),
        ]
    }

    fn intern(&mut self, shapes: &[BoundingBox]) -> Result<&'static [u16], FreezeError> {
        let mut ids = Vec::with_capacity(shapes.len());
        for shape in shapes {
            let next = COLLISION_SHAPES.len() + self.extra.len();
            let id = match self.index.get(&Self::key(shape)) {
                Some(id) => *id,
                None => {
                    let id = u16::try_from(next).map_err(|_| FreezeError::TooManyEntries {
                        registry: "collision shapes",
                    })?;
                    self.index.insert(Self::key(shape), id);
                    self.extra.push(*shape);
                    id
                }
            };
            ids.push(id);
        }
        Ok(Box::leak(ids.into_boxed_slice()))
    }
}

impl BlockTables {
    pub(crate) fn build(defs: Vec<BlockDef>, names: &NameTables) -> Result<Self, FreezeError> {
        let mut blocks = Vec::with_capacity(defs.len());
        let mut by_name = HashMap::with_capacity(defs.len());
        let mut by_item = HashMap::new();
        let mut states = Vec::new();
        let mut shapes = ShapeInterner::new();
        let mut next_state = u32::from(BlockStateId::VANILLA_COUNT);

        for (index, def) in defs.into_iter().enumerate() {
            validate_modded_name(REGISTRY, &def.name)?;
            if by_name.contains_key(def.name.as_str()) {
                return Err(FreezeError::DuplicateName {
                    registry: REGISTRY,
                    name: def.name,
                });
            }
            validate_states(&def)?;

            let raw_id = u32::from(BlockId::VANILLA_COUNT) + index as u32;
            let last_state = next_state + def.states.len() as u32;
            // Ids stay strictly below u16::MAX so a count always fits in u16.
            if raw_id >= u32::from(u16::MAX) || last_state > u32::from(u16::MAX) {
                return Err(FreezeError::TooManyEntries { registry: REGISTRY });
            }
            let block_id = BlockId::from_raw_unchecked(raw_id as u16);

            let property_names: Vec<&'static str> = def
                .properties
                .iter()
                .map(|(name, _)| leak_str(name.clone()))
                .collect();

            let mut built_states = Vec::with_capacity(def.states.len());
            let mut state_props = Vec::with_capacity(def.states.len());
            for (offset, state) in def.states.iter().enumerate() {
                let block_entity_type = match &state.block_entity_type {
                    None => u16::MAX,
                    Some(name) => resolve_block_entity_type(name, names).ok_or_else(|| {
                        FreezeError::UnknownReference {
                            registry: SyncedRegistry::BlockEntityType.id(),
                            name: name.clone(),
                            referenced_by: def.name.clone(),
                        }
                    })?,
                };
                built_states.push(BlockState {
                    id: BlockStateId::from_raw_unchecked((next_state + offset as u32) as u16),
                    state_flags: state.state_flags,
                    side_flags: state.side_flags,
                    instrument: state.instrument,
                    luminance: state.luminance,
                    piston_behavior: state.piston_behavior.clone(),
                    hardness: state.hardness,
                    collision_shapes: shapes.intern(&state.collision_shapes)?,
                    outline_shapes: shapes.intern(&state.outline_shapes)?,
                    opacity: state.opacity,
                    block_entity_type,
                });
                let props: Vec<(&'static str, &'static str)> = property_names
                    .iter()
                    .zip(&state.values)
                    .map(|(name, value)| (*name, leak_str(value.clone())))
                    .collect();
                state_props.push(&*Box::leak(props.into_boxed_slice()));
            }

            let built_states: &'static [BlockState] = Box::leak(built_states.into_boxed_slice());
            let name = leak_str(def.name);
            let block: &'static Block = Box::leak(Box::new(Block {
                id: block_id,
                name,
                hardness: def.hardness,
                blast_resistance: def.blast_resistance,
                map_color: def.map_color,
                slipperiness: def.slipperiness,
                velocity_multiplier: def.velocity_multiplier,
                jump_velocity_multiplier: def.jump_velocity_multiplier,
                item_id: def.item_id,
                default_state: &built_states[def.default_state],
                states: built_states,
                flammable: def.flammable,
                experience: def.experience,
            }));

            for ((state, props), def_state) in built_states.iter().zip(state_props).zip(&def.states)
            {
                states.push(ModdedState {
                    block: block_id,
                    state,
                    props,
                    random_ticks: def_state.random_ticks,
                });
            }
            next_state = last_state;
            if block.item_id != 0 {
                by_item.entry(block.item_id).or_insert(block);
            }
            by_name.insert(name, block);
            blocks.push(block);
        }

        Ok(Self {
            blocks: blocks.into_boxed_slice(),
            by_name,
            by_item,
            states: states.into_boxed_slice(),
            shapes: shapes.extra.into_boxed_slice(),
        })
    }
}

fn validate_states(def: &BlockDef) -> Result<(), FreezeError> {
    if def.states.is_empty() {
        return Err(invalid(&def.name, "no states"));
    }
    if def.default_state >= def.states.len() {
        return Err(invalid(&def.name, "default state out of range"));
    }
    let mut seen_properties = HashSet::new();
    for (name, values) in &def.properties {
        if values.is_empty() || !seen_properties.insert(name.as_str()) {
            return Err(invalid(&def.name, format!("bad property {name}")));
        }
    }
    let mut seen_states = HashSet::new();
    for state in &def.states {
        if state.values.len() != def.properties.len() {
            return Err(invalid(
                &def.name,
                "state value count does not match properties",
            ));
        }
        for ((name, allowed), value) in def.properties.iter().zip(&state.values) {
            if !allowed.contains(value) {
                return Err(invalid(
                    &def.name,
                    format!("value {value} not allowed for {name}"),
                ));
            }
        }
        if !seen_states.insert(state.values.as_slice()) {
            return Err(invalid(&def.name, "duplicate state"));
        }
    }
    Ok(())
}

/// Index into the block-entity type registry for a vanilla path or a modded name.
fn resolve_block_entity_type(name: &str, names: &NameTables) -> Option<u16> {
    let path = name.strip_prefix("minecraft:").unwrap_or(name);
    if let Some(index) = BLOCK_ENTITY_TYPES.iter().position(|t| *t == path) {
        return Some(index as u16);
    }
    names.id(SyncedRegistry::BlockEntityType, name)
}

impl BlockTables {
    pub(crate) fn by_name(&self, name: &str) -> Option<&'static Block> {
        self.by_name.get(name).copied()
    }
}

pub(crate) fn install(tables: BlockTables) -> Result<(), FreezeError> {
    BLOCKS.set(tables).map_err(|_| FreezeError::AlreadyFrozen)
}

/// Number of modded blocks.
#[inline]
#[must_use]
pub fn block_count() -> u16 {
    BLOCKS.get().map_or(0, |t| t.blocks.len() as u16)
}

/// Number of modded block states.
#[inline]
#[must_use]
pub fn state_count() -> u16 {
    BLOCKS.get().map_or(0, |t| t.states.len() as u16)
}

/// All modded blocks in raw-id order.
#[must_use]
pub fn blocks() -> &'static [&'static Block] {
    BLOCKS.get().map_or(&[], |t| &t.blocks)
}

/// The first modded block whose item is `item_id`.
#[must_use]
pub fn block_for_item(item_id: u16) -> Option<&'static Block> {
    BLOCKS.get()?.by_item.get(&item_id).copied()
}

/// A modded block by namespaced name.
#[must_use]
pub fn block_by_name(name: &str) -> Option<&'static Block> {
    BLOCKS.get()?.by_name.get(name).copied()
}

fn modded_state(raw: u16) -> Option<&'static ModdedState> {
    let index = raw.checked_sub(BlockStateId::VANILLA_COUNT)?;
    BLOCKS.get()?.states.get(usize::from(index))
}

/// A modded block by raw id. Falls back to air for ids that are not modded blocks, which only
/// happens if a caller forged an id; [`BlockId::new`] never produces one.
#[inline(never)]
pub(crate) fn block(raw: u16) -> &'static Block {
    raw.checked_sub(BlockId::VANILLA_COUNT)
        .and_then(|index| BLOCKS.get()?.blocks.get(usize::from(index)).copied())
        .unwrap_or(&Block::AIR)
}

/// A modded state by raw id, falling back to air like [`block`].
#[inline(never)]
pub(crate) fn state(raw: u16) -> &'static BlockState {
    modded_state(raw).map_or(Block::AIR.default_state, |s| s.state)
}

/// Block of a modded state, falling back to air like [`block`].
#[inline(never)]
pub(crate) fn block_id_of_state(raw: u16) -> BlockId {
    modded_state(raw).map_or(BlockId::AIR, |s| s.block)
}

/// Whether a modded state receives random ticks.
#[must_use]
pub fn has_random_ticks(raw: u16) -> bool {
    modded_state(raw).is_some_and(|s| s.random_ticks)
}

/// A collision or outline shape by index, including modded shapes past the vanilla table.
#[inline]
#[must_use]
pub fn shape(index: u16) -> BoundingBox {
    let index = usize::from(index);
    match COLLISION_SHAPES.get(index) {
        Some(shape) => *shape,
        None => BLOCKS
            .get()
            .and_then(|t| t.shapes.get(index - COLLISION_SHAPES.len()).copied())
            .unwrap_or(COLLISION_SHAPES[0]),
    }
}

/// Property values of a modded state, keyed by the state's id.
pub(crate) struct DynamicProperties {
    state: BlockStateId,
}

impl DynamicProperties {
    pub(crate) fn of_state(block: &Block, state: BlockStateId) -> Option<Self> {
        let modded = modded_state(state.as_u16())?;
        (modded.block == block.id && !modded.props.is_empty()).then_some(Self { state })
    }

    /// Builds properties for `block` from key/value pairs. Missing keys keep the default state's
    /// value and unknown keys are ignored, like vanilla's property parsing.
    pub(crate) fn from_pairs(block: &Block, pairs: &[(&str, &str)]) -> Option<Self> {
        let default = modded_state(block.default_state.id.as_u16())?;
        if default.props.is_empty() {
            return None;
        }
        let wanted: Vec<&str> = default
            .props
            .iter()
            .map(|(name, value)| {
                pairs
                    .iter()
                    .find(|(k, _)| k == name)
                    .map_or(*value, |(_, v)| *v)
            })
            .collect();
        let state = block.states.iter().find(|state| {
            modded_state(state.id.as_u16())
                .is_some_and(|s| s.props.iter().map(|(_, v)| *v).eq(wanted.iter().copied()))
        })?;
        Some(Self { state: state.id })
    }
}

impl BlockProperties for DynamicProperties {
    fn to_index(&self) -> u16 {
        let block = Block::from_state_id(self.state);
        self.state.as_u16() - block.states[0].id.as_u16()
    }

    fn from_index(_index: u16) -> Self {
        // Needs a block to be meaningful; only reachable through generic code on concrete types.
        Self {
            state: BlockStateId::AIR,
        }
    }

    fn handles_block_id(id: BlockId) -> bool {
        id.as_u16() >= BlockId::VANILLA_COUNT
    }

    fn to_state_id(&self, _block: &Block) -> BlockStateId {
        self.state
    }

    fn from_state_id(id: BlockStateId, _block: &Block) -> Self {
        Self { state: id }
    }

    fn default(block: &Block) -> Self {
        Self {
            state: block.default_state.id,
        }
    }

    fn to_props(&self) -> Vec<(&'static str, &'static str)> {
        modded_state(self.state.as_u16()).map_or_else(Vec::new, |s| s.props.to_vec())
    }

    fn from_props(props: &[(&str, &str)], block: &Block) -> Self {
        Self::from_pairs(block, props).unwrap_or(Self {
            state: block.default_state.id,
        })
    }
}
