//! Collision shapes of plugin blocks that depend on their block entity.
//!
//! A mod's `getShape` can read the block entity, like a Storage Drawers drawer with a hopper
//! upgrade. The plugin sets the shape with `modded.set-block-collision-shape`; the host keeps it,
//! saves it with the block entity, and uses it for entity movement, so collisions never call the
//! plugin.

use std::sync::Arc;

use pumpkin_data::{Block, BlockState};
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_nbt::tag::NbtTag;
use pumpkin_util::math::boundingbox::BoundingBox;
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector3::Vector3;
use pumpkin_world::chunk::io::Dirtiable;

use super::World;

/// The block-entity NBT key the shape is saved under. Stripped from the data plugins see.
pub const NBT_KEY: &str = "PumpkinCollision";

fn write(shapes: &[BoundingBox]) -> NbtTag {
    NbtTag::List(
        shapes
            .iter()
            .map(|b| {
                NbtTag::List(
                    [b.min.x, b.min.y, b.min.z, b.max.x, b.max.y, b.max.z]
                        .into_iter()
                        .map(NbtTag::Double)
                        .collect(),
                )
            })
            .collect(),
    )
}

fn read(nbt: &NbtCompound) -> Option<Arc<[BoundingBox]>> {
    let list = nbt.get_list(NBT_KEY)?;
    let boxes = list
        .iter()
        .filter_map(|b| {
            let NbtTag::List(v) = b else { return None };
            let v: Vec<f64> = v
                .iter()
                .filter_map(|d| match d {
                    NbtTag::Double(d) => Some(*d),
                    _ => None,
                })
                .collect();
            (v.len() == 6).then(|| BoundingBox {
                min: Vector3::new(v[0], v[1], v[2]),
                max: Vector3::new(v[3], v[4], v[5]),
            })
        })
        .collect::<Vec<_>>();
    Some(boxes.into())
}

impl World {
    /// The collision boxes a plugin set for the block at `pos`, relative to the block, or `None`
    /// to use the state's shapes. Only modded blocks can have one.
    pub fn plugin_collision_shapes(
        &self,
        pos: &BlockPos,
        state: &BlockState,
    ) -> Option<Arc<[BoundingBox]>> {
        if Block::from_state_id(state.id).is_vanilla() {
            return None;
        }
        if let Some(shapes) = self.plugin_shapes.get(pos) {
            return shapes.clone();
        }
        let shapes = self
            .pending_block_entity_nbt(pos)
            .and_then(|nbt| read(&nbt));
        self.plugin_shapes.insert(*pos, shapes.clone());
        shapes
    }

    /// `modded.set-block-collision-shape`: `None` goes back to the state's shapes.
    pub fn set_plugin_collision_shapes(&self, pos: &BlockPos, shapes: Option<Vec<BoundingBox>>) {
        let shapes: Option<Arc<[BoundingBox]>> = shapes.map(Into::into);
        self.plugin_shapes.insert(*pos, shapes.clone());
        self.level.read_chunk_sync(&pos.chunk_position(), |chunk| {
            let mut pending = chunk
                .pending_block_entities
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            if let Some(nbt) = pending.get_mut(pos) {
                match &shapes {
                    Some(shapes) => nbt.put(NBT_KEY, write(shapes)),
                    None => {
                        nbt.child_tags.remove(NBT_KEY);
                    }
                }
                drop(pending);
                chunk.mark_dirty(true);
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shapes_round_trip() {
        let shapes = [BoundingBox {
            min: Vector3::new(0.0, 0.0, 0.125),
            max: Vector3::new(1.0, 0.75, 1.0),
        }];
        let mut nbt = NbtCompound::new();
        nbt.put(NBT_KEY, write(&shapes));
        let read = read(&nbt).unwrap_or_default();
        assert_eq!(read.len(), 1);
        assert_eq!(read[0].min.z, 0.125);
        assert_eq!(read[0].max.y, 0.75);
    }
}
