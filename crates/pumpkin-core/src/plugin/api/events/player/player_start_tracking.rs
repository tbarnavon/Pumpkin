use std::sync::Arc;

use pumpkin_macros::Event;

use crate::entity::player::Player;

/// An entity became visible to a player's client (Fabric `EntityTrackingEvents.START_TRACKING`).
#[derive(Event, Clone)]
pub struct PlayerStartTrackingEvent {
    pub player: Arc<Player>,
    pub entity_id: i32,
    pub entity_type: String,
}
