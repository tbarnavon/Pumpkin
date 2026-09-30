use pumpkin_macros::Event;
use pumpkin_util::math::position::BlockPos;

/// A block entity left its world, removed or unloaded with its chunk (Fabric `ServerBlockEntityEvents.BLOCK_ENTITY_UNLOAD`).
#[derive(Event, Clone)]
pub struct BlockEntityUnloadEvent {
    pub world_name: String,
    pub block_position: BlockPos,
    pub block_entity_type: String,
}
