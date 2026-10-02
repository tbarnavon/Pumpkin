use std::sync::Arc;

use pumpkin_data::Block;
use pumpkin_macros::Event;
use pumpkin_util::math::position::BlockPos;

use crate::entity::player::Player;
use crate::world::World;

/// A `block-break-event` was cancelled (Fabric `PlayerBlockBreakEvents.CANCELED`).
#[derive(Event, Clone)]
pub struct BlockBreakCanceledEvent {
    pub player: Option<Arc<Player>>,
    pub world: Arc<World>,
    pub block: &'static Block,
    pub block_position: BlockPos,
}
