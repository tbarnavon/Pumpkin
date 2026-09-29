//! Modded items.

use std::collections::HashMap;
use std::sync::OnceLock;

use super::{FreezeError, leak_str, validate_modded_name};
use crate::data_component::DataComponent;
use crate::data_component_impl::DataComponentImpl;
use crate::item::Item;

const REGISTRY: &str = "minecraft:item";

/// A modded item as dumped from the running mod.
pub struct ItemDef {
    /// Namespaced name, for example `storagedrawers:upgrade_template`.
    pub name: String,
    /// Default components. Only component types Pumpkin knows can be listed here; modded
    /// component types are added by later phases.
    pub components: Vec<(DataComponent, Box<dyn DataComponentImpl>)>,
}

pub(crate) struct ItemTables {
    items: Box<[&'static Item]>,
    by_name: HashMap<&'static str, &'static Item>,
}

static ITEMS: OnceLock<ItemTables> = OnceLock::new();

impl ItemTables {
    pub(crate) fn build(defs: Vec<ItemDef>) -> Result<Self, FreezeError> {
        let mut items = Vec::with_capacity(defs.len());
        let mut by_name = HashMap::with_capacity(defs.len());
        for (index, def) in defs.into_iter().enumerate() {
            validate_modded_name(REGISTRY, &def.name)?;
            if by_name.contains_key(def.name.as_str()) {
                return Err(FreezeError::DuplicateName {
                    registry: REGISTRY,
                    name: def.name,
                });
            }
            let raw_id = u32::from(Item::VANILLA_COUNT) + index as u32;
            if raw_id >= u32::from(u16::MAX) {
                return Err(FreezeError::TooManyEntries { registry: REGISTRY });
            }
            let components: Vec<(DataComponent, &'static dyn DataComponentImpl)> = def
                .components
                .into_iter()
                .map(|(kind, data)| (kind, &*Box::leak(data)))
                .collect();
            let name = leak_str(def.name);
            let item: &'static Item = Box::leak(Box::new(Item {
                id: raw_id as u16,
                registry_key: name,
                components: Box::leak(components.into_boxed_slice()),
            }));
            by_name.insert(name, item);
            items.push(item);
        }
        Ok(Self {
            items: items.into_boxed_slice(),
            by_name,
        })
    }
}

impl ItemTables {
    pub(crate) fn by_name(&self, name: &str) -> Option<&'static Item> {
        self.by_name.get(name).copied()
    }
}

pub(crate) fn install(tables: ItemTables) -> Result<(), FreezeError> {
    ITEMS.set(tables).map_err(|_| FreezeError::AlreadyFrozen)
}

/// Number of modded items.
#[must_use]
pub fn item_count() -> u16 {
    ITEMS.get().map_or(0, |t| t.items.len() as u16)
}

/// All modded items in raw-id order.
#[must_use]
pub fn items() -> &'static [&'static Item] {
    ITEMS.get().map_or(&[], |t| &t.items)
}

/// A modded item by raw id.
#[must_use]
pub fn item(raw: u16) -> Option<&'static Item> {
    let index = raw.checked_sub(Item::VANILLA_COUNT)?;
    ITEMS.get()?.items.get(usize::from(index)).copied()
}

/// A modded item by namespaced name.
#[must_use]
pub fn item_by_name(name: &str) -> Option<&'static Item> {
    ITEMS.get()?.by_name.get(name).copied()
}

impl Item {
    /// Total number of items, vanilla plus modded.
    #[must_use]
    pub fn count() -> u16 {
        Self::VANILLA_COUNT + item_count()
    }

    /// Get an item by raw id.
    #[must_use]
    pub fn from_id(id: u16) -> Option<&'static Self> {
        Self::from_vanilla_id(id).or_else(|| item(id))
    }

    /// Look up an item by resource location. `minecraft:` is optional for vanilla items.
    #[must_use]
    pub fn from_registry_key(name: &str) -> Option<&'static Self> {
        match name.split_once(':') {
            None => Self::from_vanilla_key(name),
            Some(("minecraft", path)) => Self::from_vanilla_key(path),
            Some(_) => item_by_name(name),
        }
    }

    /// Whether this is a vanilla item.
    #[must_use]
    pub const fn is_vanilla(&self) -> bool {
        self.id < Self::VANILLA_COUNT
    }

    /// The namespaced name, for example `minecraft:stick` or `storagedrawers:upgrade_template`.
    #[must_use]
    pub fn namespaced_name(&self) -> std::borrow::Cow<'static, str> {
        if self.registry_key.contains(':') {
            std::borrow::Cow::Borrowed(self.registry_key)
        } else {
            std::borrow::Cow::Owned(format!("minecraft:{}", self.registry_key))
        }
    }
}
