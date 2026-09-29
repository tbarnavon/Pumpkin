//! Tag entries added by mods, on top of the generated vanilla tags.
//!
//! Mods add their own entries to vanilla tags (for example `minecraft:mineable/axe`) and define new
//! tags. The merged result replaces the vanilla tag for lookups by name, and [`has_extra`] answers
//! membership checks against a vanilla `Tag` constant.

use std::collections::{HashMap, HashSet};
use std::sync::OnceLock;

use super::{FreezeError, leak_str};
use crate::tag::{RegistryKey, get_latest_map};

/// Entries to add to one tag.
#[derive(Debug, Clone)]
pub struct TagDef {
    pub registry: RegistryKey,
    /// Full tag name, for example `minecraft:mineable/axe` or `c:chests`.
    pub name: String,
    /// Entry names (`minecraft:oak_log`, `storagedrawers:oak_trim`) or tag references
    /// (`#minecraft:logs`). Tag references are expanded against vanilla tags and the tags added
    /// before this one.
    pub entries: Vec<String>,
}

struct MergedTag {
    values: &'static [&'static str],
    ids: &'static [u16],
    /// Ids that are not in the vanilla tag of the same name.
    extra: HashSet<u16>,
}

pub(crate) struct TagTables {
    tags: HashMap<RegistryKey, HashMap<&'static str, MergedTag>>,
}

static TAGS: OnceLock<TagTables> = OnceLock::new();

/// Resolves an entry name to `(raw id, stored value)`: the stored value is the bare path for
/// vanilla entries and the namespaced name for modded ones, matching the generated tags.
pub(crate) type Resolver<'a> = dyn Fn(RegistryKey, &str) -> Option<(u16, &'static str)> + 'a;

impl TagTables {
    pub(crate) fn build(defs: Vec<TagDef>, resolve: &Resolver<'_>) -> Result<Self, FreezeError> {
        let mut working: HashMap<(RegistryKey, String), (Vec<&'static str>, Vec<u16>)> =
            HashMap::new();

        for def in defs {
            let key = (def.registry, def.name.clone());
            let (mut values, mut ids) = working.remove(&key).unwrap_or_else(|| {
                get_latest_map(def.registry)
                    .get(def.name.as_str())
                    .map(|t| (t.0.to_vec(), t.1.to_vec()))
                    .unwrap_or_default()
            });
            for entry in &def.entries {
                if let Some(reference) = entry.strip_prefix('#') {
                    let referenced = working
                        .get(&(def.registry, reference.to_string()))
                        .map(|(v, i)| (v.clone(), i.clone()))
                        .or_else(|| {
                            get_latest_map(def.registry)
                                .get(reference)
                                .map(|t| (t.0.to_vec(), t.1.to_vec()))
                        })
                        .ok_or_else(|| FreezeError::UnknownReference {
                            registry: "tag",
                            name: entry.clone(),
                            referenced_by: def.name.clone(),
                        })?;
                    for (value, id) in referenced.0.into_iter().zip(referenced.1) {
                        if !ids.contains(&id) {
                            values.push(value);
                            ids.push(id);
                        }
                    }
                    continue;
                }
                let (id, value) =
                    resolve(def.registry, entry).ok_or_else(|| FreezeError::UnknownReference {
                        registry: "tag",
                        name: entry.clone(),
                        referenced_by: def.name.clone(),
                    })?;
                if !ids.contains(&id) {
                    values.push(value);
                    ids.push(id);
                }
            }
            working.insert(key, (values, ids));
        }

        let mut tags: HashMap<RegistryKey, HashMap<&'static str, MergedTag>> = HashMap::new();
        for ((registry, name), (values, ids)) in working {
            let vanilla: &[u16] = get_latest_map(registry)
                .get(name.as_str())
                .map_or(&[], |t| t.1);
            let extra = ids
                .iter()
                .copied()
                .filter(|id| !vanilla.contains(id))
                .collect();
            let merged = MergedTag {
                values: Box::leak(values.into_boxed_slice()),
                ids: Box::leak(ids.into_boxed_slice()),
                extra,
            };
            tags.entry(registry)
                .or_default()
                .insert(leak_str(name), merged);
        }
        Ok(Self { tags })
    }
}

pub(crate) fn install(tables: TagTables) -> Result<(), FreezeError> {
    TAGS.set(tables).map_err(|_| FreezeError::AlreadyFrozen)
}

fn get(registry: RegistryKey, tag: &str) -> Option<&'static MergedTag> {
    TAGS.get()?.tags.get(&registry)?.get(tag)
}

/// Merged entry names of a tag that mods added to or created.
#[must_use]
pub fn values(registry: RegistryKey, tag: &str) -> Option<&'static [&'static str]> {
    get(registry, tag).map(|t| t.values)
}

/// Merged raw ids of a tag that mods added to or created.
#[must_use]
pub fn ids(registry: RegistryKey, tag: &str) -> Option<&'static [u16]> {
    get(registry, tag).map(|t| t.ids)
}

/// Whether a mod added `id` to the tag named `tag`. Vanilla membership is checked by the caller.
#[inline]
#[must_use]
pub fn has_extra(registry: RegistryKey, tag: &str, id: u16) -> bool {
    match TAGS.get() {
        None => false,
        Some(tables) => tables
            .tags
            .get(&registry)
            .and_then(|tags| tags.get(tag))
            .is_some_and(|t| t.extra.contains(&id)),
    }
}

/// Every tag mods added to or created, with merged contents, for sending to clients.
pub fn modded_tags(registry: RegistryKey) -> impl Iterator<Item = (&'static str, &'static [u16])> {
    TAGS.get()
        .and_then(|tables| tables.tags.get(&registry))
        .into_iter()
        .flat_map(|tags| tags.iter())
        .map(|(name, tag)| (*name, tag.ids))
}
