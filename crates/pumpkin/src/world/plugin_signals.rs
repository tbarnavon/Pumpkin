//! Redstone and comparator outputs of plugin blocks, set by the plugin and read by the host
//! (`world.set-redstone-output`, `world.set-comparator-output`), so redstone never calls the
//! plugin.

use std::sync::Arc;

use pumpkin_nbt::compound::NbtCompound;
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::chunk::io::Dirtiable;

use super::World;

/// The block-entity NBT key the outputs are saved under. Stripped from the data plugins see.
pub const NBT_KEY: &str = "PumpkinSignals";

#[derive(Clone, Copy, Default, PartialEq, Eq, Debug)]
pub struct PluginSignals {
    pub weak: u8,
    pub strong: u8,
    pub comparator: u8,
}

impl PluginSignals {
    fn write(self) -> NbtCompound {
        let mut nbt = NbtCompound::new();
        nbt.put_byte("weak", self.weak as i8);
        nbt.put_byte("strong", self.strong as i8);
        nbt.put_byte("comparator", self.comparator as i8);
        nbt
    }

    fn read(nbt: &NbtCompound) -> Self {
        let get = |key| nbt.get_byte(key).map_or(0, |v| (v as u8).min(15));
        Self {
            weak: get("weak"),
            strong: get("strong"),
            comparator: get("comparator"),
        }
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

    /// `world.set-redstone-output`: stores the power and updates the neighbours, and their
    /// neighbours when the strong power changed, as vanilla does for a block that starts or
    /// stops powering what it touches.
    pub fn set_plugin_redstone_output(self: &Arc<Self>, pos: &BlockPos, weak: u8, strong: u8) {
        let (weak, strong) = (weak.min(15), strong.min(15));
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
            for direction in pumpkin_data::BlockDirection::all() {
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
        let signals = PluginSignals {
            weak: 7,
            strong: 3,
            comparator: 15,
        };
        assert_eq!(PluginSignals::read(&signals.write()), signals);
        let mut nbt = NbtCompound::new();
        nbt.put_byte("weak", 40);
        assert_eq!(PluginSignals::read(&nbt).weak, 15);
    }
}
