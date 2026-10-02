use std::sync::Arc;

use pumpkin_macros::Event;
use pumpkin_util::math::position::BlockPos;

use crate::world::World;

/// Fired once per world tick when hoppers moved items in or out of plugin item storages
/// (`world.set-item-storage`). The storages already hold the new contents.
#[derive(Event, Clone)]
pub struct ItemStorageChangedEvent {
    pub world: Arc<World>,
    /// The block positions whose storage changed.
    pub positions: Vec<BlockPos>,
}
