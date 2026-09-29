//! `fabric:registry/sync` and `fabric:registry/sync/complete`.
//!
//! fabric-registry-sync-v0 `impl/registry/sync/packet/RegistrySyncPayload.java`, `write` at
//! line 122 and `read` at line 76:
//!
//! ```text
//! VarInt  registry namespace group count
//!   String  namespace ("" stands for "minecraft", optimizeNamespace at :215)
//!   VarInt  registry count
//!     String  registry path
//!     byte    attributes (bit 0 = OPTIONAL, encodeRegistryAttributes at :194)
//!     VarInt  entry namespace group count
//!       String  namespace ("" stands for "minecraft")
//!       VarInt  bulk count
//!         VarInt  first raw id - last raw id of the previous bulk (0 before the first bulk of
//!                 the registry; carried across namespace groups)
//!         VarInt  bulk size
//!         bulk size x String entry path (consecutive raw ids)
//! ```

use super::{ReadError, Reader, write_string, write_var_int};

/// `RegistrySyncPayload.ID` (`RegistrySyncPayload.java:59`).
pub const SYNC_CHANNEL: &str = "fabric:registry/sync";
/// `SyncCompletePayload.ID`: serverbound, empty body.
pub const SYNC_COMPLETE_CHANNEL: &str = "fabric:registry/sync/complete";

const OPTIONAL: u8 = 0x1;
/// Upper bound on list lengths read back, well above any real registry.
const MAX_ENTRIES: usize = 1 << 20;

/// One synced registry: its id and every entry with its raw id, in the order to send them.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SyncedRegistry {
    /// For example `minecraft:block`.
    pub id: String,
    pub optional: bool,
    /// `(namespaced entry id, raw id)`.
    pub entries: Vec<(String, u32)>,
}

fn split(id: &str) -> (&str, &str) {
    id.split_once(':').unwrap_or(("minecraft", id))
}

fn optimize_namespace(namespace: &str) -> &str {
    if namespace == "minecraft" {
        ""
    } else {
        namespace
    }
}

const fn unoptimize_namespace(namespace: &str) -> &str {
    if namespace.is_empty() {
        "minecraft"
    } else {
        namespace
    }
}

/// Groups `items` by key, keeping the order in which each key first appears.
fn group_by_first_occurrence<'a, T>(
    items: impl Iterator<Item = (&'a str, T)>,
) -> Vec<(&'a str, Vec<T>)> {
    let mut groups: Vec<(&str, Vec<T>)> = Vec::new();
    for (key, item) in items {
        match groups.iter_mut().find(|(k, _)| *k == key) {
            Some((_, group)) => group.push(item),
            None => groups.push((key, vec![item])),
        }
    }
    groups
}

/// Encodes the payload body.
///
/// Registry and entry namespace groups keep the order of first appearance. Java groups registry namespaces through a `HashMap`; with every synced registry in
/// `minecraft` that makes no difference, and the client never depends on group order anyway.
#[must_use]
pub fn encode(registries: &[SyncedRegistry]) -> Vec<u8> {
    let mut out = Vec::new();
    let registry_groups = group_by_first_occurrence(registries.iter().map(|r| (split(&r.id).0, r)));
    write_var_int(&mut out, registry_groups.len() as i32);
    for (namespace, group) in registry_groups {
        write_string(&mut out, optimize_namespace(namespace));
        write_var_int(&mut out, group.len() as i32);
        for registry in group {
            write_string(&mut out, split(&registry.id).1);
            out.push(if registry.optional { OPTIONAL } else { 0 });

            let entry_groups = group_by_first_occurrence(
                registry
                    .entries
                    .iter()
                    .map(|(id, raw)| (split(id).0, (split(id).1, *raw))),
            );
            write_var_int(&mut out, entry_groups.len() as i32);
            let mut last_bulk_last_raw_id = 0u32;
            for (namespace, mut entries) in entry_groups {
                entries.sort_by_key(|(_, raw)| *raw);
                let mut bulks: Vec<&[(&str, u32)]> = Vec::new();
                let mut start = 0;
                for i in 1..=entries.len() {
                    if i == entries.len() || entries[i - 1].1 + 1 != entries[i].1 {
                        bulks.push(&entries[start..i]);
                        start = i;
                    }
                }
                write_string(&mut out, optimize_namespace(namespace));
                write_var_int(&mut out, bulks.len() as i32);
                for bulk in bulks {
                    write_var_int(
                        &mut out,
                        bulk[0].1.wrapping_sub(last_bulk_last_raw_id) as i32,
                    );
                    write_var_int(&mut out, bulk.len() as i32);
                    for (path, raw) in bulk {
                        write_string(&mut out, path);
                        last_bulk_last_raw_id = *raw;
                    }
                }
            }
        }
    }
    out
}

/// Decodes a payload body, like `RegistrySyncPayload.read`.
pub fn decode(payload: &[u8]) -> Result<Vec<SyncedRegistry>, ReadError> {
    let mut reader = Reader::new(payload);
    let mut registries = Vec::new();
    let group_count = reader.length(MAX_ENTRIES)?;
    for _ in 0..group_count {
        let namespace = unoptimize_namespace(reader.string()?).to_string();
        let registry_count = reader.length(MAX_ENTRIES)?;
        for _ in 0..registry_count {
            let path = reader.string()?;
            let attributes = reader.byte()?;
            let mut entries = Vec::new();
            let mut last_bulk_last_raw_id = 0u32;
            let entry_group_count = reader.length(MAX_ENTRIES)?;
            for _ in 0..entry_group_count {
                let entry_namespace = unoptimize_namespace(reader.string()?).to_string();
                let bulk_count = reader.length(MAX_ENTRIES)?;
                for _ in 0..bulk_count {
                    let diff = reader.var_int()? as u32;
                    let size = reader.length(MAX_ENTRIES)?;
                    let mut raw = last_bulk_last_raw_id.wrapping_add(diff).wrapping_sub(1);
                    for _ in 0..size {
                        raw = raw.wrapping_add(1);
                        entries.push((format!("{entry_namespace}:{}", reader.string()?), raw));
                    }
                    last_bulk_last_raw_id = raw;
                }
            }
            registries.push(SyncedRegistry {
                id: format!("{namespace}:{path}"),
                optional: attributes & OPTIONAL != 0,
                entries,
            });
        }
    }
    if !reader.is_empty() {
        return Err(ReadError::BadLength(reader.rest().len() as i32));
    }
    Ok(registries)
}
