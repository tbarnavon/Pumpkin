//! Redstone and comparator outputs of plugin blocks.
//!
//! The plugin sets them (`world.set-redstone-output`, `set-redstone-output-sides`,
//! `set-comparator-output`) and the host reads them, so redstone never calls the plugin.

use std::sync::Arc;

use pumpkin_data::BlockDirection;
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_nbt::tag::NbtTag;
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::chunk::io::Dirtiable;

use super::World;

/// The block-entity NBT key the outputs are saved under. Stripped from the data plugins see.
pub const NBT_KEY: &str = "PumpkinSignals";

/// The power a plugin block gives, per side in `BlockDirection` order (the direction of the
/// query, as vanilla's `getSignal(..., side)`), and what comparators read.
#[derive(Clone, Copy, Default, PartialEq, Eq, Debug)]
pub struct PluginSignals {
    pub weak: [u8; 6],
    pub strong: [u8; 6],
    pub comparator: u8,
}

/// One power per side, saved as a byte when every side is the same.
fn write_sides(nbt: &mut NbtCompound, key: &str, sides: [u8; 6]) {
    if sides.iter().all(|v| *v == sides[0]) {
        nbt.put_byte(key, sides[0] as i8);
    } else {
        nbt.put(key, NbtTag::ByteArray(sides.map(|v| v as i8).into()));
    }
}

fn read_sides(nbt: &NbtCompound, key: &str) -> [u8; 6] {
    let clamp = |v: i8| (v as u8).min(15);
    if let Some(array) = nbt.get_byte_array(key) {
        let mut sides = [0; 6];
        for (side, v) in sides.iter_mut().zip(array) {
            *side = clamp(*v);
        }
        return sides;
    }
    [nbt.get_byte(key).map_or(0, clamp); 6]
}

impl PluginSignals {
    fn write(self) -> NbtCompound {
        let mut nbt = NbtCompound::new();
        write_sides(&mut nbt, "weak", self.weak);
        write_sides(&mut nbt, "strong", self.strong);
        nbt.put_byte("comparator", self.comparator as i8);
        nbt
    }

    fn read(nbt: &NbtCompound) -> Self {
        Self {
            weak: read_sides(nbt, "weak"),
            strong: read_sides(nbt, "strong"),
            comparator: nbt.get_byte("comparator").map_or(0, |v| (v as u8).min(15)),
        }
    }

    /// The weak power asked with `direction`.
    #[must_use]
    pub const fn weak(&self, direction: BlockDirection) -> u8 {
        self.weak[direction.to_index() as usize]
    }

    /// The strong power asked with `direction`.
    #[must_use]
    pub const fn strong(&self, direction: BlockDirection) -> u8 {
        self.strong[direction.to_index() as usize]
    }
}

impl World {
    /// The outputs set on the plugin block at `pos`.
    pub fn plugin_signals(&self, pos: &BlockPos) -> PluginSignals {
        if let Some(signals) = self.plugin_signals.get(pos) {
            return *signals;
        }
        let signals = self
            .pending_block_entity_nbt(pos)
            .and_then(|nbt| nbt.get_compound(NBT_KEY).map(PluginSignals::read));
        if let Some(signals) = signals {
            self.plugin_signals.insert(*pos, signals);
        }
        signals.unwrap_or_default()
    }

    fn store_plugin_signals(&self, pos: &BlockPos, signals: PluginSignals) {
        self.plugin_signals.insert(*pos, signals);
        self.level.read_chunk_sync(&pos.chunk_position(), |chunk| {
            let mut pending = chunk
                .pending_block_entities
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            if let Some(nbt) = pending.get_mut(pos) {
                nbt.put_compound(NBT_KEY, signals.write());
                drop(pending);
                chunk.mark_dirty(true);
            }
        });
    }

    /// `world.set-redstone-output` and `set-redstone-output-sides`: stores the power per side
    /// and updates the neighbours, and their neighbours when the strong power changed, as
    /// vanilla does for a block that starts or stops powering what it touches.
    pub fn set_plugin_redstone_output(
        self: &Arc<Self>,
        pos: &BlockPos,
        weak: [u8; 6],
        strong: [u8; 6],
    ) {
        let (weak, strong) = (weak.map(|v| v.min(15)), strong.map(|v| v.min(15)));
        let old = self.plugin_signals(pos);
        if old.weak == weak && old.strong == strong {
            return;
        }
        self.store_plugin_signals(
            pos,
            PluginSignals {
                weak,
                strong,
                ..old
            },
        );
        let block = self.get_block(pos);
        self.update_neighbors_at(pos, block, None);
        if old.strong != strong {
            for direction in BlockDirection::all() {
                self.update_neighbors_at(&pos.offset(direction.to_offset()), block, None);
            }
        }
    }

    /// `world.set-comparator-output`: stores the value and updates comparators reading it.
    pub fn set_plugin_comparator_output(self: &Arc<Self>, pos: &BlockPos, value: u8) {
        let value = value.min(15);
        let old = self.plugin_signals(pos);
        if old.comparator == value {
            return;
        }
        self.store_plugin_signals(
            pos,
            PluginSignals {
                comparator: value,
                ..old
            },
        );
        self.update_neighbour_for_output_signal(pos, self.get_block(pos));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn signals_round_trip_and_clamp() {
        let mut strong = [0; 6];
        strong[BlockDirection::Up.to_index() as usize] = 3;
        let signals = PluginSignals {
            weak: [7; 6],
            strong,
            comparator: 15,
        };
        let nbt = signals.write();
        assert!(nbt.get_byte("weak").is_some());
        assert!(nbt.get_byte_array("strong").is_some());
        assert_eq!(PluginSignals::read(&nbt), signals);
        assert_eq!(PluginSignals::read(&nbt).strong(BlockDirection::Up), 3);
        let mut nbt = NbtCompound::new();
        nbt.put_byte("weak", 40);
        assert_eq!(PluginSignals::read(&nbt).weak, [15; 6]);
    }
}
