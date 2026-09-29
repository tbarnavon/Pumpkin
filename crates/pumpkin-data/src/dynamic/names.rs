//! Registries where Pumpkin only needs a name and a raw id for modded entries: their behaviour is
//! supplied elsewhere (plugins), but the ids must match the client's.

use std::collections::HashSet;
use std::sync::OnceLock;

use super::{FreezeError, leak_str, validate_modded_name};

/// A synced registry that accepts modded entries by name.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SyncedRegistry {
    BlockEntityType,
    EntityType,
    Menu,
    DataComponentType,
}

impl SyncedRegistry {
    pub const ALL: [Self; 4] = [
        Self::BlockEntityType,
        Self::EntityType,
        Self::Menu,
        Self::DataComponentType,
    ];

    /// The registry's identifier, as used by the protocol and Fabric registry sync.
    #[must_use]
    pub const fn id(self) -> &'static str {
        match self {
            Self::BlockEntityType => "minecraft:block_entity_type",
            Self::EntityType => "minecraft:entity_type",
            Self::Menu => "minecraft:menu",
            Self::DataComponentType => "minecraft:data_component_type",
        }
    }

    const fn index(self) -> usize {
        self as usize
    }

    /// Number of vanilla entries; modded entries are numbered from here on.
    #[must_use]
    pub fn vanilla_len(self) -> u16 {
        match self {
            #[cfg(feature = "block")]
            Self::BlockEntityType => crate::block_properties::BLOCK_ENTITY_TYPES.len() as u16,
            #[cfg(feature = "entity")]
            Self::EntityType => crate::entity::EntityType::ALL.len() as u16,
            #[cfg(feature = "screen")]
            Self::Menu => crate::screen::WindowType::VANILLA_NAMES.len() as u16,
            #[cfg(feature = "data_component")]
            Self::DataComponentType => {
                let mut count = 0u16;
                while crate::data_component::DataComponent::try_from_id(count as u8).is_some() {
                    count += 1;
                }
                count
            }
            #[allow(unreachable_patterns)]
            _ => 0,
        }
    }

    /// Whether `name` is a vanilla entry of this registry. Accepts bare paths and
    /// `minecraft:`-prefixed names.
    #[must_use]
    pub fn is_vanilla_name(self, name: &str) -> bool {
        let path = name.strip_prefix("minecraft:").unwrap_or(name);
        match self {
            #[cfg(feature = "block")]
            Self::BlockEntityType => crate::block_properties::BLOCK_ENTITY_TYPES.contains(&path),
            #[cfg(feature = "entity")]
            Self::EntityType => crate::entity::EntityType::from_name(path).is_some(),
            #[cfg(feature = "screen")]
            Self::Menu => crate::screen::WindowType::VANILLA_NAMES.contains(&path),
            #[cfg(feature = "data_component")]
            Self::DataComponentType => {
                let mut id = 0u8;
                while let Some(component) = crate::data_component::DataComponent::try_from_id(id) {
                    if component.to_name() == name
                        || component.to_name().strip_prefix("minecraft:") == Some(path)
                    {
                        return true;
                    }
                    id += 1;
                }
                false
            }
            #[allow(unreachable_patterns)]
            _ => false,
        }
    }
}

/// Modded entries of every [`SyncedRegistry`], indexed by `raw id - vanilla_len`.
pub(crate) struct NameTables {
    entries: [Vec<&'static str>; 4],
    vanilla_len: [u16; 4],
}

static NAMES: OnceLock<NameTables> = OnceLock::new();

impl NameTables {
    pub(crate) fn build(entries: &[(SyncedRegistry, String)]) -> Result<Self, FreezeError> {
        let mut tables = Self {
            entries: Default::default(),
            vanilla_len: SyncedRegistry::ALL.map(SyncedRegistry::vanilla_len),
        };
        let mut seen: HashSet<(SyncedRegistry, &str)> = HashSet::new();
        for (registry, name) in entries {
            validate_modded_name(registry.id(), name)?;
            if !seen.insert((*registry, name.as_str())) || registry.is_vanilla_name(name) {
                return Err(FreezeError::DuplicateName {
                    registry: registry.id(),
                    name: name.clone(),
                });
            }
            let table = &mut tables.entries[registry.index()];
            if usize::from(tables.vanilla_len[registry.index()]) + table.len()
                >= usize::from(u16::MAX)
            {
                return Err(FreezeError::TooManyEntries {
                    registry: registry.id(),
                });
            }
            table.push(leak_str(name.clone()));
        }
        Ok(tables)
    }

    /// Raw id of a modded entry.
    pub(crate) fn id(&self, registry: SyncedRegistry, name: &str) -> Option<u16> {
        let index = self.entries[registry.index()]
            .iter()
            .position(|n| *n == name)?;
        Some(self.vanilla_len[registry.index()] + index as u16)
    }
}

pub(crate) fn install(tables: NameTables) -> Result<(), FreezeError> {
    NAMES.set(tables).map_err(|_| FreezeError::AlreadyFrozen)
}

/// Whether the overlay has been installed.
#[must_use]
pub fn is_frozen() -> bool {
    NAMES.get().is_some()
}

/// Modded entries of `registry` in raw-id order, starting at [`SyncedRegistry::vanilla_len`].
#[must_use]
pub fn modded_entries(registry: SyncedRegistry) -> &'static [&'static str] {
    NAMES
        .get()
        .map_or(&[], |tables| tables.entries[registry.index()].as_slice())
}

/// Name of a modded entry by raw id, or `None` for vanilla or unknown ids.
#[must_use]
pub fn modded_name(registry: SyncedRegistry, raw_id: u16) -> Option<&'static str> {
    let tables = NAMES.get()?;
    let index = raw_id.checked_sub(tables.vanilla_len[registry.index()])?;
    tables.entries[registry.index()]
        .get(usize::from(index))
        .copied()
}

/// Raw id of a modded entry by its namespaced name.
#[must_use]
pub fn modded_id(registry: SyncedRegistry, name: &str) -> Option<u16> {
    NAMES.get()?.id(registry, name)
}

/// Raw id of a block-entity type by resource location: vanilla (`minecraft:chest` or `chest`) or
/// modded (`storagedrawers:standard_drawers_1`).
#[cfg(feature = "block")]
#[must_use]
pub fn block_entity_type_id(name: &str) -> Option<u16> {
    match name.split_once(':') {
        None | Some(("minecraft", _)) => {
            let path = name.strip_prefix("minecraft:").unwrap_or(name);
            crate::block_properties::BLOCK_ENTITY_TYPES
                .iter()
                .position(|t| *t == path)
                .map(|index| index as u16)
        }
        Some(_) => modded_id(SyncedRegistry::BlockEntityType, name),
    }
}

/// Namespaced name of a block-entity type by raw id, vanilla or modded.
#[cfg(feature = "block")]
#[must_use]
pub fn block_entity_type_name(raw_id: u16) -> Option<std::borrow::Cow<'static, str>> {
    match crate::block_properties::BLOCK_ENTITY_TYPES.get(usize::from(raw_id)) {
        Some(path) => Some(std::borrow::Cow::Owned(format!("minecraft:{path}"))),
        None => {
            modded_name(SyncedRegistry::BlockEntityType, raw_id).map(std::borrow::Cow::Borrowed)
        }
    }
}
