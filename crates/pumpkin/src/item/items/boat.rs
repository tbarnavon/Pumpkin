use std::sync::Arc;

use crate::entity::Entity;
use crate::entity::EntityBase;
use crate::entity::player::Player;
use crate::entity::vehicle::boat::{BOAT_TYPES, BoatEntity};
use crate::item::{ItemBehaviour, ItemMetadata};
use pumpkin_data::Block;
use pumpkin_data::entity::EntityType;
use pumpkin_data::fluid::Fluid;
use pumpkin_data::item::Item;
use pumpkin_util::math::boundingbox::{BoundingBox, EntityDimensions};
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector3::Vector3;

use crate::world::World;

pub struct BoatItem;

impl BoatItem {
    /// The entity a boat item places, `boat` or `chest_boat`, and its type (an index into
    /// [`BOAT_TYPES`]).
    pub(crate) fn item_to_entity(item: &Item) -> (&'static EntityType, usize) {
        for (index, (_, boat, chest_boat)) in BOAT_TYPES.iter().enumerate() {
            if item.id == boat.id {
                return (&EntityType::BOAT, index);
            }
            if item.id == chest_boat.id {
                return (&EntityType::CHEST_BOAT, index);
            }
        }
        tracing::error!("Unknown boat item ID: {}", item.id);
        (&EntityType::BOAT, 0)
    }

    /// Gets entity dimensions for the boat type
    const fn get_entity_dimensions(entity_type: &EntityType) -> EntityDimensions {
        EntityDimensions::new(
            entity_type.dimension[0],
            entity_type.dimension[1],
            entity_type.eye_height,
        )
    }
}

impl ItemMetadata for BoatItem {
    fn ids() -> Box<[u16]> {
        BOAT_TYPES
            .iter()
            .flat_map(|(_, boat, chest_boat)| [boat.id, chest_boat.id])
            .collect()
    }
}

impl ItemBehaviour for BoatItem {
    /// Vanilla: `BoatItem.use()` - raycasts to find placement position
    fn normal_use(&self, item: &Item, player: &Player) {
        let world = player.world();
        let (start_pos, end_pos) = self.get_start_and_end_pos(player);

        // Vanilla: raycast with FluidHandling.ANY - stops on water/lava surface or solid blocks
        let checker = |pos: &BlockPos, world_inner: &Arc<World>| {
            let state_id = world_inner.get_block_state_id(pos);

            // Air doesn't stop the raycast
            if state_id == Block::AIR.default_state.id {
                return false;
            }

            // Check if it's a fluid - stop on any fluid (FluidHandling.ANY in vanilla)
            if Fluid::from_state_id(state_id).is_some() {
                return true;
            }

            // Stop on solid blocks
            true
        };

        let Some((hit_pos, _direction)) = world.raycast(start_pos, end_pos, checker) else {
            return;
        };

        // Calculate hit position - center of the block top or water surface
        // TODO: Vanilla uses exact raycast intersection point (hitResult.getPos()),
        // Pumpkin's raycast only returns block positions.
        let hit_vec = Vector3::new(
            f64::from(hit_pos.0.x) + 0.5,
            f64::from(hit_pos.0.y) + 1.0,
            f64::from(hit_pos.0.z) + 0.5,
        );

        // Vanilla: Check for entities in the path that would block placement
        // Get player's rotation vector stretched by 5.0 and expanded by 1.0
        let (yaw, pitch) = player.rotation();
        let rotation_vec =
            Vector3::rotation_vector(f64::from(pitch), f64::from(yaw)).multiply(5.0, 5.0, 5.0);
        let player_entity = player.get_entity().bounding_box.load();
        let search_box = player_entity.stretch(rotation_vec).expand_all(1.0);

        let entities = world.get_entities_at_box(&search_box);
        let player_eye_pos = player.eye_position();

        for entity in &entities {
            // Vanilla: EntityPredicates.CAN_HIT = !isSpectator() && canHit()
            if entity.is_spectator() || !entity.can_hit() {
                continue;
            }

            // Vanilla: expand by getTargetingMargin() (0.0 for most entities)
            let entity_box = entity.get_entity().bounding_box.load();
            // Check if entity's bounding box contains the player's eye position
            if entity_box.intersects(&BoundingBox::new(player_eye_pos, player_eye_pos)) {
                // Entity is blocking the line of sight at eye position
                return;
            }
        }

        // Create the boat entity
        let (entity_type, boat_type) = Self::item_to_entity(item);
        let dimensions = Self::get_entity_dimensions(entity_type);
        let boat_box = BoundingBox::new_from_pos(hit_vec.x, hit_vec.y, hit_vec.z, &dimensions);

        // Vanilla: if (!world.isSpaceEmpty(lv7, lv7.getBoundingBox())) return FAIL
        if !world.is_space_empty(boat_box) {
            return;
        }

        // Check no entities occupy the space
        if !world.get_entities_at_box(&boat_box).is_empty() {
            return;
        }

        // Create and spawn the boat
        let entity = Entity::new(world.clone(), hit_vec, entity_type);

        // Set yaw to player's yaw
        let (player_yaw, _) = player.rotation();
        entity.set_rotation(player_yaw, 0.0);

        let boat_entity = Arc::new(BoatEntity::new(entity));
        boat_entity.set_boat_type(boat_type);
        world.spawn_entity(boat_entity);

        let mut main_hand = player.inventory.held_item();
        let consumed = if !main_hand.is_empty() && main_hand.item.id == item.id {
            main_hand.decrement_unless_creative(player.gamemode.load(), 1);
            player.inventory.set_held_item(main_hand);
            true
        } else {
            false
        };

        if !consumed {
            let mut off_hand = player.inventory.off_hand_item();
            if !off_hand.is_empty() && off_hand.item.id == item.id {
                off_hand.decrement_unless_creative(player.gamemode.load(), 1);
                player
                    .inventory
                    .set_stack_in_hand(pumpkin_util::Hand::Left, off_hand);
            }
        }
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
