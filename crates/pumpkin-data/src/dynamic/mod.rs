//! Registry entries added at startup after the vanilla ones, for example the blocks and items of
//! a Fabric mod.
//!
//! Vanilla data stays in the generated static tables and keeps its ids. Modded entries are
//! collected in a [`DynamicRegistriesBuilder`] and installed once with
//! [`DynamicRegistriesBuilder::freeze`], before any world is loaded. After that the overlay is
//! read-only: lookups past the vanilla range go to the leaked `'static` data built here.
//!
//! Modded ids are assigned in insertion order, directly after the vanilla ids of each registry.
//! Fabric's registry sync rebuilds block-state ids on the client by walking blocks in raw-id order
//! and appending each block's possible states (`StateIdTracker.recalcStateMap`), so blocks and
//! their states must be added in the order the mod registered them.

#[cfg(feature = "block")]
pub mod blocks;
#[cfg(feature = "item")]
pub mod items;
pub mod names;
#[cfg(feature = "tag")]
pub mod tags;

use std::fmt;

/// Collects modded registry entries before they are frozen into the global overlay.
#[derive(Default)]
pub struct DynamicRegistriesBuilder {
    #[cfg(feature = "block")]
    blocks: Vec<blocks::BlockDef>,
    #[cfg(feature = "item")]
    items: Vec<items::ItemDef>,
    #[cfg(feature = "tag")]
    tags: Vec<tags::TagDef>,
    entries: Vec<(names::SyncedRegistry, String)>,
}

/// Why the overlay could not be installed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FreezeError {
    /// `freeze` was called before; the overlay can only be installed once per process.
    AlreadyFrozen,
    /// Two entries in the same registry share a name, or a modded name shadows a vanilla one.
    DuplicateName {
        registry: &'static str,
        name: String,
    },
    /// A name is not namespaced (`namespace:path`) or uses the `minecraft` namespace.
    InvalidName {
        registry: &'static str,
        name: String,
    },
    /// A registry would grow past the `u16` id space the protocol code uses.
    TooManyEntries { registry: &'static str },
    /// A block definition is internally inconsistent.
    InvalidBlock { name: String, reason: String },
    /// A referenced entry does not exist.
    UnknownReference {
        registry: &'static str,
        name: String,
        referenced_by: String,
    },
}

impl fmt::Display for FreezeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::AlreadyFrozen => write!(f, "dynamic registries are already frozen"),
            Self::DuplicateName { registry, name } => {
                write!(f, "duplicate entry {name} in {registry}")
            }
            Self::InvalidName { registry, name } => write!(
                f,
                "invalid name {name} in {registry}: expected namespace:path outside minecraft"
            ),
            Self::TooManyEntries { registry } => write!(f, "too many entries in {registry}"),
            Self::InvalidBlock { name, reason } => write!(f, "invalid block {name}: {reason}"),
            Self::UnknownReference {
                registry,
                name,
                referenced_by,
            } => write!(
                f,
                "{referenced_by} references unknown {registry} entry {name}"
            ),
        }
    }
}

impl std::error::Error for FreezeError {}

impl DynamicRegistriesBuilder {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds a block. Blocks get raw ids in the order they are added, and their states follow in
    /// the order of [`blocks::BlockDef::states`].
    #[cfg(feature = "block")]
    pub fn add_block(&mut self, block: blocks::BlockDef) -> &mut Self {
        self.blocks.push(block);
        self
    }

    /// Adds an item. Items get raw ids in the order they are added.
    #[cfg(feature = "item")]
    pub fn add_item(&mut self, item: items::ItemDef) -> &mut Self {
        self.items.push(item);
        self
    }

    /// Adds an entry to a registry that only needs a name and a raw id, such as
    /// `minecraft:block_entity_type` or `minecraft:menu`.
    pub fn add_entry(
        &mut self,
        registry: names::SyncedRegistry,
        name: impl Into<String>,
    ) -> &mut Self {
        self.entries.push((registry, name.into()));
        self
    }

    /// Adds entries to a tag, creating the tag if it does not exist yet. Entries may be vanilla or
    /// modded names; unknown names fail the freeze.
    #[cfg(feature = "tag")]
    pub fn add_tag_entries(&mut self, tag: tags::TagDef) -> &mut Self {
        self.tags.push(tag);
        self
    }

    /// Returns `true` when nothing was added.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        #[cfg(feature = "block")]
        let blocks = self.blocks.is_empty();
        #[cfg(not(feature = "block"))]
        let blocks = true;
        #[cfg(feature = "item")]
        let items = self.items.is_empty();
        #[cfg(not(feature = "item"))]
        let items = true;
        #[cfg(feature = "tag")]
        let tags = self.tags.is_empty();
        #[cfg(not(feature = "tag"))]
        let tags = true;
        self.entries.is_empty() && blocks && items && tags
    }

    /// Validates everything, assigns ids and installs the overlay for the rest of the process.
    ///
    /// Nothing is installed if this returns an error. Must run before any modded id is looked up;
    /// vanilla lookups work before and after.
    pub fn freeze(self) -> Result<(), FreezeError> {
        // Names first: blocks refer to block-entity types by name.
        let names = names::NameTables::build(&self.entries)?;
        #[cfg(feature = "item")]
        let items = items::ItemTables::build(self.items)?;
        #[cfg(feature = "block")]
        let blocks = blocks::BlockTables::build(self.blocks, &names)?;
        #[cfg(feature = "tag")]
        let tags = {
            let resolve =
                |registry: crate::tag::RegistryKey, name: &str| -> Option<(u16, &'static str)> {
                    let modded = name
                        .split_once(':')
                        .is_some_and(|(ns, _)| ns != "minecraft");
                    match registry {
                        #[cfg(feature = "block")]
                        crate::tag::RegistryKey::Block => {
                            let block = if modded {
                                blocks.by_name(name)
                            } else {
                                crate::Block::from_name(name)
                            }?;
                            Some((block.id.as_u16(), block.name))
                        }
                        #[cfg(feature = "item")]
                        crate::tag::RegistryKey::Item => {
                            let item = if modded {
                                items.by_name(name)
                            } else {
                                crate::item::Item::from_registry_key(name)
                            }?;
                            Some((item.id, item.registry_key))
                        }
                        _ => None,
                    }
                };
            tags::TagTables::build(self.tags, &resolve)?
        };

        // Check-then-set is fine: freeze runs once, during startup, on one thread. Each `set`
        // below can only fail if another thread raced us here, which would be a startup bug.
        if names::is_frozen() {
            return Err(FreezeError::AlreadyFrozen);
        }
        names::install(names)?;
        #[cfg(feature = "item")]
        items::install(items)?;
        #[cfg(feature = "block")]
        blocks::install(blocks)?;
        #[cfg(feature = "tag")]
        tags::install(tags)?;
        Ok(())
    }
}

/// Checks that `name` is `namespace:path`, not in the `minecraft` namespace, and made of the
/// characters vanilla allows in identifiers.
pub(crate) fn validate_modded_name(registry: &'static str, name: &str) -> Result<(), FreezeError> {
    let invalid = || FreezeError::InvalidName {
        registry,
        name: name.to_string(),
    };
    let (namespace, path) = name.split_once(':').ok_or_else(invalid)?;
    let namespace_ok = !namespace.is_empty()
        && namespace
            .bytes()
            .all(|b| matches!(b, b'a'..=b'z' | b'0'..=b'9' | b'_' | b'-' | b'.'));
    let path_ok = !path.is_empty()
        && path
            .bytes()
            .all(|b| matches!(b, b'a'..=b'z' | b'0'..=b'9' | b'_' | b'-' | b'.' | b'/'));
    if !namespace_ok || !path_ok || namespace == "minecraft" {
        return Err(invalid());
    }
    Ok(())
}

/// Leaks a string so it can live in `'static` registry data. Only called while freezing.
pub(crate) fn leak_str(s: String) -> &'static str {
    Box::leak(s.into_boxed_str())
}
