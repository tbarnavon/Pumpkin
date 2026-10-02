use std::sync::Arc;

use pumpkin_data::item_stack::ItemStack;
use pumpkin_macros::{Event, cancellable};

use crate::entity::player::Player;

/// A player middle-clicks an entity (Fabric `PlayerPickItemEvents.ENTITY`). Set `item` to give that stack instead of the spawn egg; cancel to give nothing.
#[cancellable]
#[derive(Event, Clone)]
pub struct PlayerPickItemEntityEvent {
    pub player: Arc<Player>,
    pub entity_id: i32,
    pub entity_type: String,
    pub include_data: bool,
    /// Set by a plugin: the stack to pick instead.
    pub item: Option<ItemStack>,
}
