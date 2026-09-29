//! Keeps palette entries whose block is not registered, so a world survives a restart without
//! the mod that added them.
//!
//! On load such an entry becomes [`STAND_IN`] and its original palette compound is remembered per
//! position in [`ChunkData::unknown_blocks`](crate::chunk::ChunkData). On save every position that
//! still holds the stand-in gets its original compound back. Nothing unknown ever reaches the
//! network or gameplay code; they only see the stand-in.

use pumpkin_data::{Block, BlockStateId};
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_nbt::tag::NbtTag;
use pumpkin_util::math::position::BlockPos;
use rustc_hash::FxHashMap;

use crate::chunk::palette::{BLOCK_DISK_MIN_BITS, BlockPalette, PalettedContainer};
use crate::generation::structure::template::{BlockStateResolver, PaletteEntry};

/// What an unknown block looks like while its mod is missing: visible, and unbreakable in
/// survival so players cannot destroy the saved data by accident.
pub const STAND_IN: BlockStateId = Block::BEDROCK.default_state.id;

/// Original palette compounds of unknown blocks, by absolute position.
pub type UnknownBlocks = FxHashMap<BlockPos, NbtCompound>;

/// A resolved section palette plus the unknown entries in it, by palette index.
type DecodedPalette = (Box<[BlockStateId]>, Vec<(u16, NbtCompound)>);

const SECTION_SIZE: usize = 16;

/// Resolves a disk palette. Compound entries that name an unregistered block become
/// [`STAND_IN`] and are returned with their palette index.
pub(super) fn decode_block_palette(tag: &NbtTag) -> Option<DecodedPalette> {
    let NbtTag::List(list) = tag else {
        return super::extract_u16_array(tag).map(|palette| (palette, Vec::new()));
    };
    if !list
        .iter()
        .all(|entry| matches!(entry, NbtTag::Compound(_)))
    {
        return super::extract_u16_array(tag).map(|palette| (palette, Vec::new()));
    }

    let mut palette = Vec::with_capacity(list.len());
    let mut unknown = Vec::new();
    for (index, entry) in list.iter().enumerate() {
        let NbtTag::Compound(compound) = entry else {
            continue;
        };
        let resolved = PaletteEntry::from_nbt_compound(compound)
            .ok()
            .and_then(|entry| {
                Block::from_name(&entry.name)?;
                BlockStateResolver::resolve_simple(&entry)
            });
        if let Some(state) = resolved {
            palette.push(state.id);
        } else {
            palette.push(STAND_IN);
            // Vanilla palettes hold at most 4096 entries; a corrupt one past u16 is not preserved.
            if let Ok(index) = u16::try_from(index) {
                unknown.push((index, compound.clone()));
            }
        }
    }
    Some((palette.into_boxed_slice(), unknown))
}

/// Records the absolute position of every block in a section that uses an unknown palette entry.
pub(super) fn record_positions(
    out: &mut UnknownBlocks,
    chunk_x: i32,
    chunk_z: i32,
    section_y: i32,
    palette_len: usize,
    data: Option<&[i64]>,
    unknown: &[(u16, NbtCompound)],
) {
    let keys: Vec<u16> = (0..palette_len as u16).collect();
    let indices = PalettedContainer::<u16, SECTION_SIZE>::from_palette_and_packed_data(
        &keys,
        data.unwrap_or_default(),
        BLOCK_DISK_MIN_BITS,
    );
    for y in 0..SECTION_SIZE {
        for z in 0..SECTION_SIZE {
            for x in 0..SECTION_SIZE {
                let key = indices.get(x, y, z);
                if let Some((_, compound)) = unknown.iter().find(|(index, _)| *index == key) {
                    let pos = BlockPos::new(
                        chunk_x * 16 + x as i32,
                        section_y * 16 + y as i32,
                        chunk_z * 16 + z as i32,
                    );
                    out.insert(pos, compound.clone());
                }
            }
        }
    }
}

/// Disk palette compound for a registered state.
pub(super) fn state_palette_entry(id: BlockStateId) -> NbtCompound {
    let block = Block::from_state_id(id);
    let mut comp = NbtCompound::new();
    comp.put_string("Name", block.namespaced_name().into_owned());
    if let Some(props) = block.properties(id) {
        let prop_vec = props.to_props();
        if !prop_vec.is_empty() {
            let mut props_comp = NbtCompound::new();
            for (k, v) in prop_vec {
                props_comp.put_string(k, v.to_string());
            }
            comp.put_compound("Properties", props_comp);
        }
    }
    comp
}

/// Palette key while re-encoding a section: a registered state, or an unknown entry by index.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Default)]
enum Key {
    #[default]
    Unset,
    State(BlockStateId),
    Unknown(u16),
}

/// Encodes a section that still contains unknown blocks. Returns the palette and packed data in
/// the vanilla disk layout, with the original compounds restored where the stand-in remains.
///
/// `unknown` lists positions inside the section (x, y, z) and their original compounds; entries
/// whose position no longer holds [`STAND_IN`] are ignored.
pub(super) fn encode_section(
    blocks: &BlockPalette,
    unknown: &[((usize, usize, usize), &NbtCompound)],
) -> (Vec<NbtTag>, Option<Vec<i64>>) {
    let mut compounds: Vec<&NbtCompound> = Vec::new();
    let mut keys = PalettedContainer::<Key, SECTION_SIZE>::default();
    for y in 0..SECTION_SIZE {
        for z in 0..SECTION_SIZE {
            for x in 0..SECTION_SIZE {
                let state = blocks.get(x, y, z);
                let original = (state == STAND_IN)
                    .then(|| unknown.iter().find(|(pos, _)| *pos == (x, y, z)))
                    .flatten();
                let key = match original {
                    Some((_, compound)) => {
                        let index =
                            compounds
                                .iter()
                                .position(|c| c == compound)
                                .unwrap_or_else(|| {
                                    compounds.push(compound);
                                    compounds.len() - 1
                                });
                        Key::Unknown(index as u16)
                    }
                    None => Key::State(state),
                };
                keys.set(x, y, z, key);
            }
        }
    }

    let (palette, packed) = keys.to_block_disk_palette_and_packed_data();
    let palette = palette
        .iter()
        .map(|key| {
            NbtTag::Compound(match key {
                Key::State(id) => state_palette_entry(*id),
                Key::Unknown(index) => compounds[usize::from(*index)].clone(),
                Key::Unset => state_palette_entry(BlockStateId::AIR),
            })
        })
        .collect();
    let data = (!packed.is_empty()).then(|| packed.into_vec());
    (palette, data)
}
