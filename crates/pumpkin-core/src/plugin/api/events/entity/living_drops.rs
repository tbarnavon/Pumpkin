use std::sync::Arc;

use pumpkin_data::damage::DamageType;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_macros::{Event, cancellable};
use pumpkin_util::math::vector3::Vector3;

use crate::world::World;

/// What a dying living entity drops (NeoForge `LivingDropsEvent` and
/// `LivingExperienceDropEvent`).
///
/// Handlers may replace `drops` and change `experience`; cancelling drops no items.
#[cancellable]
#[derive(Event, Clone)]
pub struct LivingDropsEvent {
    pub entity_id: i32,
    pub entity_type: String,
    pub world: Arc<World>,
    pub position: Vector3<f64>,
    pub damage_type: DamageType,
    /// `LivingEntity.getKillCredit`.
    pub killer: Option<(i32, String)>,
    /// `lastHurtByPlayerMemoryTime > 0`.
    pub recently_hit: bool,
    pub drops: Vec<ItemStack>,
    pub experience: i32,
}
