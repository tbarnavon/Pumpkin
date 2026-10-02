use std::sync::Arc;

use pumpkin_data::item_stack::ItemStack;
use pumpkin_macros::{Event, cancellable};
use pumpkin_util::math::position::BlockPos;

use crate::entity::player::Player;

/// A player middle-clicks a block (Fabric `PlayerPickItemEvents.BLOCK`). Set `item` to give that stack instead of the block's item; cancel to give nothing.
#[cancellable]
#[derive(Event, Clone)]
pub struct PlayerPickItemBlockEvent {
    pub player: Arc<Player>,
    pub block_position: BlockPos,
    pub state_id: u16,
    pub include_data: bool,
    /// Set by a plugin: the stack to pick instead.
    pub item: Option<ItemStack>,
}
