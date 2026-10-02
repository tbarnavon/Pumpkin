use std::sync::Arc;

use pumpkin_macros::Event;

use crate::entity::player::Player;

/// An entity stopped being visible to a player's client (Fabric `EntityTrackingEvents.STOP_TRACKING`).
#[derive(Event, Clone)]
pub struct PlayerStopTrackingEvent {
    pub player: Arc<Player>,
    pub entity_id: i32,
    pub entity_type: String,
}
