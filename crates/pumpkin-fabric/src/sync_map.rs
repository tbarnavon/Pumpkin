//! Builds the `fabric:registry/sync` contents from Pumpkin's registries.
//!
//! Fabric's server sends every registry that is both SYNCED and MODDED, with all of its entries,
//! vanilla ones included (`RegistrySyncManager.createAndPopulateRegistryMap`, lines 160-260).
//! A registry becomes MODDED once a mod adds an entry, so here: once the overlay has entries.

use pumpkin_data::block_properties::BLOCK_ENTITY_TYPES;
use pumpkin_data::data_component::DataComponent;
use pumpkin_data::dynamic::names::{self, SyncedRegistry as NamedRegistry};
use pumpkin_data::entity::EntityType;
use pumpkin_data::item::Item;
use pumpkin_data::screen::WindowType;
use pumpkin_data::{Block, BlockId};

use crate::wire::registry_sync::SyncedRegistry;

/// The registries to sync, in the order a Fabric 26.3 server sends them (the iteration order of
/// `BuiltInRegistries.REGISTRY` there). The client does not depend on this order; keeping it
/// makes Pumpkin's payload byte-identical to Fabric's for the same content.
#[derive(Clone, Copy)]
enum Synced {
    Item,
    BlockEntityType,
    DataComponentType,
    Menu,
    Block,
    EntityType,
}

const ORDER: [Synced; 6] = [
    Synced::Item,
    Synced::BlockEntityType,
    Synced::DataComponentType,
    Synced::Menu,
    Synced::Block,
    Synced::EntityType,
];

fn namespaced(path: &str) -> String {
    if path.contains(':') {
        path.to_string()
    } else {
        format!("minecraft:{path}")
    }
}

fn named(registry: NamedRegistry, vanilla: impl Iterator<Item = String>) -> Option<Vec<String>> {
    let modded = names::modded_entries(registry);
    if modded.is_empty() {
        return None;
    }
    Some(
        vanilla
            .chain(modded.iter().map(|name| (*name).to_string()))
            .collect(),
    )
}

fn entries(registry: Synced) -> Option<(&'static str, Vec<String>)> {
    Some(match registry {
        Synced::Item => {
            if Item::count() == Item::VANILLA_COUNT {
                return None;
            }
            let names = (0..Item::count())
                .map(|raw| {
                    Item::from_id(raw)
                        .map_or_else(String::new, |item| item.namespaced_name().into_owned())
                })
                .collect();
            ("minecraft:item", names)
        }
        Synced::Block => {
            if BlockId::count() == BlockId::VANILLA_COUNT {
                return None;
            }
            let names = (0..BlockId::count())
                .filter_map(BlockId::new)
                .map(|id| Block::from_id(id).namespaced_name().into_owned())
                .collect();
            ("minecraft:block", names)
        }
        Synced::BlockEntityType => (
            NamedRegistry::BlockEntityType.id(),
            named(
                NamedRegistry::BlockEntityType,
                BLOCK_ENTITY_TYPES.iter().map(|path| namespaced(path)),
            )?,
        ),
        Synced::Menu => (
            NamedRegistry::Menu.id(),
            named(
                NamedRegistry::Menu,
                WindowType::VANILLA_NAMES
                    .iter()
                    .map(|path| namespaced(path)),
            )?,
        ),
        Synced::DataComponentType => (
            NamedRegistry::DataComponentType.id(),
            named(
                NamedRegistry::DataComponentType,
                (0..=u8::MAX)
                    .map_while(DataComponent::try_from_id)
                    .map(|component| component.to_name().to_string()),
            )?,
        ),
        Synced::EntityType => (
            NamedRegistry::EntityType.id(),
            named(
                NamedRegistry::EntityType,
                (0..EntityType::ALL.len() as u16)
                    .map_while(EntityType::from_raw)
                    .map(|entity| namespaced(entity.resource_name)),
            )?,
        ),
    })
}

/// The registries to send, or an empty list when no mod added anything (then nothing is sent).
#[must_use]
pub fn build() -> Vec<SyncedRegistry> {
    ORDER
        .into_iter()
        .filter_map(entries)
        .map(|(id, names)| SyncedRegistry {
            id: id.to_string(),
            optional: false,
            entries: names
                .into_iter()
                .zip(0u32..)
                .filter(|(name, _)| !name.is_empty())
                .collect(),
        })
        .collect()
}
