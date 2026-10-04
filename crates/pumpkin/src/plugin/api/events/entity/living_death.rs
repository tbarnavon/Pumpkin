use std::sync::Arc;

use pumpkin_data::damage::DamageType;
use pumpkin_macros::{Event, cancellable};
use pumpkin_util::math::vector3::Vector3;

use crate::world::World;

/// A living entity took fatal damage (NeoForge `LivingDeathEvent`, Fabric
/// `ServerLivingEntityEvents.ALLOW_DEATH` and `AFTER_DEATH`).
///
/// Cancelling keeps the entity alive only if a handler raised its health above 0.
#[cancellable]
#[derive(Event, Clone)]
pub struct LivingDeathEvent {
    pub entity_id: i32,
    pub entity_type: String,
    pub world: Arc<World>,
    pub position: Vector3<f64>,
    pub damage_type: DamageType,
    /// `DamageSource.getDirectEntity`.
    pub direct_entity_id: Option<i32>,
    /// `LivingEntity.getKillCredit`.
    pub killer: Option<(i32, String)>,
}
