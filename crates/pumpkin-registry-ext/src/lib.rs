//! Loads modded registry entries from Extractor mod dumps into Pumpkin's registry overlay
//! (`pumpkin_data::dynamic`).
//!
//! This crate is loader-agnostic: it knows the dump format and Pumpkin's registries, not Fabric.
//! Every dump records the raw ids a real server with the same mods assigned. Entries are added in
//! that order and the ids Pumpkin ends up with are checked against it, so a mismatch fails at
//! startup instead of desyncing clients later.

pub mod dump;

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use base64::Engine;
use pumpkin_data::block_properties::{EnumVariants, NoteblockInstrument};
use pumpkin_data::block_state::PistonBehavior;
use pumpkin_data::data_component::DataComponent;
use pumpkin_data::data_component_impl::read_data;
use pumpkin_data::dynamic::blocks::{BlockDef, StateDef};
use pumpkin_data::dynamic::items::ItemDef;
use pumpkin_data::dynamic::names::{SyncedRegistry, modded_id};
use pumpkin_data::dynamic::tags::TagDef;
use pumpkin_data::dynamic::{DynamicRegistriesBuilder, FreezeError};
use pumpkin_data::item::Item;
use pumpkin_data::tag::RegistryKey;
use pumpkin_data::{Block, BlockId, BlockStateId, Flammable};
use pumpkin_util::math::boundingbox::BoundingBox;
use pumpkin_util::math::experience::Experience;
use pumpkin_util::math::int_provider::{IntProvider, NormalIntProvider, UniformIntProvider};
use pumpkin_util::math::vector3::Vector3;
use tracing::debug;

use crate::dump::{BlockDump, ItemDump, ModDump, NamedEntry, ShapeDump, StateDump};

/// Why mod data could not be loaded.
#[derive(Debug, thiserror::Error)]
pub enum LoadError {
    #[error("cannot read {}: {source}", path.display())]
    Io {
        path: PathBuf,
        source: std::io::Error,
    },
    #[error("invalid mod dump {}: {source}", path.display())]
    Json {
        path: PathBuf,
        source: serde_json::Error,
    },
    #[error(
        "{registry}: expected raw id {expected} next but the dumps have {found}; dump all mods together from one server"
    )]
    Gap {
        registry: &'static str,
        expected: u32,
        found: u16,
    },
    #[error("{what} {value} in {context} is not supported")]
    BadValue {
        what: &'static str,
        value: String,
        context: String,
    },
    #[error("{registry} entry {name} got id {actual}, the dump says {expected}")]
    Mismatch {
        registry: &'static str,
        name: String,
        expected: u16,
        actual: u16,
    },
    #[error(transparent)]
    Freeze(#[from] FreezeError),
}

/// Data from the dumps that other parts of the server consume after the registries are frozen.
#[derive(Debug, Default)]
pub struct InstalledMods {
    pub namespaces: Vec<String>,
    /// Recipe id and JSON, for every recipe the mods define.
    pub recipes: Vec<(String, serde_json::Value)>,
    /// Loot table id and JSON, for every loot table the mods define.
    pub loot_tables: Vec<(String, serde_json::Value)>,
}

static INSTALLED: OnceLock<InstalledMods> = OnceLock::new();

/// What [`install`] loaded, or `None` when no mods were installed.
#[must_use]
pub fn installed() -> Option<&'static InstalledMods> {
    INSTALLED.get()
}

/// Reads every `*.json` mod dump in `dir`, sorted by file name. A missing directory means no mods.
pub fn read_dumps(dir: &Path) -> Result<Vec<ModDump>, LoadError> {
    let io = |source| LoadError::Io {
        path: dir.to_path_buf(),
        source,
    };
    if !dir.exists() {
        return Ok(Vec::new());
    }
    let mut paths: Vec<PathBuf> = fs::read_dir(dir)
        .map_err(io)?
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.extension().is_some_and(|ext| ext == "json"))
        .collect();
    paths.sort();
    paths
        .into_iter()
        .map(|path| {
            let text = fs::read_to_string(&path).map_err(|source| LoadError::Io {
                path: path.clone(),
                source,
            })?;
            serde_json::from_str(&text).map_err(|source| LoadError::Json { path, source })
        })
        .collect()
}

/// Installs the entries of `dumps` into the registry overlay and checks the resulting ids.
///
/// Must run once, before any world is loaded. Does nothing when `dumps` is empty, so a server
/// without mods stays exactly vanilla and [`installed`] keeps returning `None`.
pub fn install(dumps: Vec<ModDump>) -> Result<Option<&'static InstalledMods>, LoadError> {
    let mut installed = InstalledMods::default();
    if dumps.is_empty() {
        return Ok(None);
    }

    let mut blocks = Vec::new();
    let mut items = Vec::new();
    let mut entries: Vec<(SyncedRegistry, NamedEntry)> = Vec::new();
    let mut tags = Vec::new();
    for dump in dumps {
        installed.namespaces.push(dump.namespace);
        blocks.extend(dump.blocks);
        items.extend(dump.items);
        let named = [
            (SyncedRegistry::BlockEntityType, dump.block_entity_types),
            (SyncedRegistry::Menu, dump.menus),
            (SyncedRegistry::DataComponentType, dump.data_component_types),
            (SyncedRegistry::EntityType, dump.entity_types),
            (SyncedRegistry::RecipeSerializer, dump.recipe_serializers),
        ];
        for (registry, list) in named {
            entries.extend(list.into_iter().map(|entry| (registry, entry)));
        }
        for (registry, list) in [
            (RegistryKey::Block, dump.tags.block),
            (RegistryKey::Item, dump.tags.item),
        ] {
            for (name, entries) in list {
                tags.push(TagDef {
                    registry,
                    name,
                    entries,
                });
            }
        }
        installed.recipes.extend(dump.recipes);
        installed.loot_tables.extend(dump.loot_tables);
    }

    blocks.sort_by_key(|block| block.raw_id);
    items.sort_by_key(|item| item.raw_id);
    entries.sort_by_key(|(registry, entry)| (registry.id(), entry.raw_id));
    check_contiguous(
        "minecraft:block",
        BlockId::VANILLA_COUNT,
        blocks.iter().map(|b| b.raw_id),
    )?;
    check_contiguous(
        "minecraft:block states",
        BlockStateId::VANILLA_COUNT,
        blocks
            .iter()
            .flat_map(|b| b.states.iter().map(|s| s.raw_id)),
    )?;
    check_contiguous(
        "minecraft:item",
        Item::VANILLA_COUNT,
        items.iter().map(|i| i.raw_id),
    )?;
    for registry in SyncedRegistry::ALL {
        check_contiguous(
            registry.id(),
            registry.vanilla_len(),
            entries
                .iter()
                .filter(|(r, _)| *r == registry)
                .map(|(_, entry)| entry.raw_id),
        )?;
    }

    let mut builder = DynamicRegistriesBuilder::new();
    for (registry, entry) in &entries {
        builder.add_entry(*registry, entry.name.clone());
    }
    for item in &items {
        builder.add_item(item_def(item)?);
    }
    for block in &blocks {
        builder.add_block(block_def(block, &items)?);
    }
    for tag in tags {
        builder.add_tag_entries(tag);
    }
    builder.freeze()?;

    verify(&blocks, &items, &entries)?;
    // `freeze` succeeded, so this is the first install in this process.
    Ok(Some(INSTALLED.get_or_init(|| installed)))
}

fn check_contiguous(
    registry: &'static str,
    first: u16,
    ids: impl Iterator<Item = u16>,
) -> Result<(), LoadError> {
    for (expected, id) in (u32::from(first)..).zip(ids) {
        if u32::from(id) != expected {
            return Err(LoadError::Gap {
                registry,
                expected,
                found: id,
            });
        }
    }
    Ok(())
}

fn item_def(item: &ItemDump) -> Result<ItemDef, LoadError> {
    let bad_nbt = || LoadError::BadValue {
        what: "components_nbt",
        value: String::from("<invalid>"),
        context: item.name.clone(),
    };
    let bytes = base64::engine::general_purpose::STANDARD
        .decode(&item.components_nbt)
        .map_err(|_| bad_nbt())?;
    let mut cursor = std::io::Cursor::new(bytes.as_slice());
    let nbt = pumpkin_nbt::Nbt::read(&mut pumpkin_nbt::deserializer::NbtReadHelperJava::new(
        &mut cursor,
    ))
    .map_err(|_| bad_nbt())?;

    let mut components = Vec::new();
    for (name, tag) in &nbt.root_tag.child_tags {
        // Modded component types and components Pumpkin cannot read yet only matter for the
        // server's own view of default stacks; the client already has these defaults.
        let Some(kind) = DataComponent::try_from_name(name) else {
            debug!("{}: skipping component {name} (unknown type)", item.name);
            continue;
        };
        if let Some(data) = read_data(kind, tag) {
            components.push((kind, data));
        } else {
            debug!(
                "{}: skipping component {name} (unsupported value)",
                item.name
            );
        }
    }
    Ok(ItemDef {
        name: item.name.clone(),
        components,
    })
}

fn block_def(block: &BlockDump, items: &[ItemDump]) -> Result<BlockDef, LoadError> {
    let item_id = if block.item == "minecraft:air" {
        0
    } else if let Some(item) = items.iter().find(|item| item.name == block.item) {
        item.raw_id
    } else {
        Item::from_registry_key(&block.item)
            .map(|item| item.id)
            .ok_or_else(|| LoadError::BadValue {
                what: "item",
                value: block.item.clone(),
                context: block.name.clone(),
            })?
    };
    let states = block
        .states
        .iter()
        .map(|state| state_def(state, &block.name))
        .collect::<Result<Vec<_>, _>>()?;
    Ok(BlockDef {
        name: block.name.clone(),
        hardness: block.hardness,
        blast_resistance: block.blast_resistance,
        map_color: block.map_color,
        slipperiness: block.slipperiness,
        velocity_multiplier: block.velocity_multiplier,
        jump_velocity_multiplier: block.jump_velocity_multiplier,
        item_id,
        flammable: block.flammable.as_ref().map(|f| Flammable {
            spread_chance: f.spread_chance,
            burn_chance: f.burn_chance,
        }),
        experience: block.experience.as_ref().map(|xp| Experience {
            experience: if xp.min_inclusive == xp.max_inclusive {
                IntProvider::Constant(xp.min_inclusive)
            } else {
                IntProvider::Object(NormalIntProvider::Uniform(UniformIntProvider {
                    min_inclusive: xp.min_inclusive,
                    max_inclusive: xp.max_inclusive,
                }))
            },
        }),
        properties: block
            .properties
            .iter()
            .map(|p| (p.name.clone(), p.values.clone()))
            .collect(),
        states,
        default_state: block.default_state,
    })
}

fn state_def(state: &StateDump, block: &str) -> Result<StateDef, LoadError> {
    let instrument = (0..NoteblockInstrument::variant_count())
        .map(NoteblockInstrument::from_index)
        .find(|i| i.to_value() == state.instrument)
        .ok_or_else(|| LoadError::BadValue {
            what: "instrument",
            value: state.instrument.clone(),
            context: block.to_string(),
        })?;
    // Same mapping as pumpkin-codegen's PistonBehavior (vanilla PushReaction names).
    let piston_behavior = match state.piston_behavior.as_str() {
        "NORMAL" | "PUSH_PULL" => PistonBehavior::Normal,
        "DESTROY" | "POPPED" => PistonBehavior::Destroy,
        "BLOCK" => PistonBehavior::Block,
        "IGNORE" | "IMMOVEABLE" => PistonBehavior::Ignore,
        "PUSH_ONLY" | "PUSH" => PistonBehavior::PushOnly,
        other => {
            return Err(LoadError::BadValue {
                what: "piston behavior",
                value: other.to_string(),
                context: block.to_string(),
            });
        }
    };
    Ok(StateDef {
        values: state.values.clone(),
        state_flags: state.state_flags,
        side_flags: state.side_flags,
        instrument,
        luminance: state.luminance,
        piston_behavior,
        hardness: state.hardness,
        opacity: state.opacity,
        collision_shapes: state.collision_shapes.iter().map(shape).collect(),
        outline_shapes: state.outline_shapes.iter().map(shape).collect(),
        block_entity_type: state.block_entity_type.clone(),
        random_ticks: state.random_ticks,
    })
}

const fn shape(shape: &ShapeDump) -> BoundingBox {
    BoundingBox::new(
        Vector3::new(shape.min[0], shape.min[1], shape.min[2]),
        Vector3::new(shape.max[0], shape.max[1], shape.max[2]),
    )
}

fn verify(
    blocks: &[BlockDump],
    items: &[ItemDump],
    entries: &[(SyncedRegistry, NamedEntry)],
) -> Result<(), LoadError> {
    let mismatch =
        |registry: &'static str, name: &str, expected: u16, actual: u16| LoadError::Mismatch {
            registry,
            name: name.to_string(),
            expected,
            actual,
        };
    for dump in blocks {
        let block = Block::from_name(&dump.name)
            .ok_or_else(|| mismatch("minecraft:block", &dump.name, dump.raw_id, u16::MAX))?;
        if block.id.as_u16() != dump.raw_id {
            return Err(mismatch(
                "minecraft:block",
                &dump.name,
                dump.raw_id,
                block.id.as_u16(),
            ));
        }
        for (state, dump_state) in block.states.iter().zip(&dump.states) {
            if state.id.as_u16() != dump_state.raw_id {
                return Err(mismatch(
                    "minecraft:block state",
                    &dump.name,
                    dump_state.raw_id,
                    state.id.as_u16(),
                ));
            }
        }
    }
    for dump in items {
        let actual = Item::from_registry_key(&dump.name).map_or(u16::MAX, |item| item.id);
        if actual != dump.raw_id {
            return Err(mismatch("minecraft:item", &dump.name, dump.raw_id, actual));
        }
    }
    for (registry, entry) in entries {
        let actual = modded_id(*registry, &entry.name).unwrap_or(u16::MAX);
        if actual != entry.raw_id {
            return Err(mismatch(registry.id(), &entry.name, entry.raw_id, actual));
        }
    }
    Ok(())
}
