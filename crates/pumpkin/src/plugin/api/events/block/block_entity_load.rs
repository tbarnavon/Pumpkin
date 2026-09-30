use pumpkin_macros::Event;
use pumpkin_util::math::position::BlockPos;

/// A block entity was added to its world, placed or read from its chunk (Fabric `ServerBlockEntityEvents.BLOCK_ENTITY_LOAD`). Pumpkin creates block entities the first time they are used.
#[derive(Event, Clone)]
pub struct BlockEntityLoadEvent {
    pub world_name: String,
    pub block_position: BlockPos,
    pub block_entity_type: String,
}
