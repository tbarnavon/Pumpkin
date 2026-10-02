use std::sync::Arc;

use pumpkin_macros::Event;
use pumpkin_util::math::position::BlockPos;

use crate::entity::player::Player;

/// A player tries to sleep; set `time-ok` or `monsters-ok` to allow or forbid it (Fabric `EntitySleepEvents.ALLOW_SLEEP_TIME` and `ALLOW_NEARBY_MONSTERS`).
#[derive(Event, Clone)]
pub struct PlayerSleepCheckEvent {
    pub player: Arc<Player>,
    pub bed_position: BlockPos,
    pub time_ok: bool,
    pub monsters_ok: bool,
}
