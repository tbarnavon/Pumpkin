use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

use crossbeam::atomic::AtomicCell;

use crate::entity::player::Player;
use crate::entity::{Entity, EntityBase, living::LivingEntity};
use crate::server::Server;

use pumpkin_data::damage::DamageType;
use pumpkin_data::entity::EntityType;
use pumpkin_data::item::Item;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_protocol::codec::var_int::VarInt;
use pumpkin_protocol::java::client::play::Metadata;

use pumpkin_util::math::vector3::Vector3;

use crate::entity::vehicle::vehicle::VehicleEntity;

/// 1.21.1's boat types (`Boat.Type`), in order: the name saved as `Type`, the boat item and the
/// chest boat item.
pub const BOAT_TYPES: [(&str, &Item, &Item); 9] = [
    ("oak", &Item::OAK_BOAT, &Item::OAK_CHEST_BOAT),
    ("spruce", &Item::SPRUCE_BOAT, &Item::SPRUCE_CHEST_BOAT),
    ("birch", &Item::BIRCH_BOAT, &Item::BIRCH_CHEST_BOAT),
    ("jungle", &Item::JUNGLE_BOAT, &Item::JUNGLE_CHEST_BOAT),
    ("acacia", &Item::ACACIA_BOAT, &Item::ACACIA_CHEST_BOAT),
    ("cherry", &Item::CHERRY_BOAT, &Item::CHERRY_CHEST_BOAT),
    ("dark_oak", &Item::DARK_OAK_BOAT, &Item::DARK_OAK_CHEST_BOAT),
    ("mangrove", &Item::MANGROVE_BOAT, &Item::MANGROVE_CHEST_BOAT),
    ("bamboo", &Item::BAMBOO_RAFT, &Item::BAMBOO_CHEST_RAFT),
];

pub struct BoatEntity {
    pub vehicle: VehicleEntity,
    ticks_underwater: AtomicCell<f32>,
    left_paddle_moving: AtomicBool,
    right_paddle_moving: AtomicBool,
    /// Index into [`BOAT_TYPES`].
    boat_type: AtomicUsize,
}

impl BoatEntity {
    pub fn new(entity: Entity) -> Self {
        let boat = Self {
            vehicle: VehicleEntity::new(entity),
            ticks_underwater: AtomicCell::new(0.0),
            left_paddle_moving: AtomicBool::new(false),
            right_paddle_moving: AtomicBool::new(false),
            boat_type: AtomicUsize::new(0),
        };
        boat.set_boat_type(0);
        boat
    }

    const fn is_chest_boat(&self) -> bool {
        self.vehicle.entity.entity_type.id == EntityType::CHEST_BOAT.id
    }

    /// Sets the boat's type, an index into [`BOAT_TYPES`], and the item it drops.
    pub fn set_boat_type(&self, boat_type: usize) {
        let boat_type = boat_type.min(BOAT_TYPES.len() - 1);
        self.boat_type.store(boat_type, Ordering::Relaxed);
        let (_, boat, chest_boat) = BOAT_TYPES[boat_type];
        self.vehicle.drop_item.store(Some(if self.is_chest_boat() {
            chest_boat
        } else {
            boat
        }));
    }

    pub fn set_paddles(&self, left: bool, right: bool) {
        self.left_paddle_moving.store(left, Ordering::Relaxed);
        self.right_paddle_moving.store(right, Ordering::Relaxed);

        self.vehicle.entity.send_meta_data(
            &[
                Metadata::new(pumpkin_data::tracked_data::boat::ID_PADDLE_LEFT, left),
                Metadata::new(pumpkin_data::tracked_data::boat::ID_PADDLE_RIGHT, right),
            ],
            None,
        );
    }

    fn send_wobble_metadata(&self) {
        self.vehicle.send_wobble_metadata();
    }
}

impl EntityBase for BoatEntity {
    fn get_entity(&self) -> &Entity {
        &self.vehicle.entity
    }

    fn get_living_entity(&self) -> Option<&LivingEntity> {
        None
    }

    fn tick(&self, _caller: &dyn EntityBase, _server: &Server) {
        self.vehicle.tick();

        let underwater = self.ticks_underwater.load();
        if self.vehicle.entity.touching_water.load(Ordering::Relaxed) {
            self.ticks_underwater.store((underwater + 1.0).min(60.0));
        } else if underwater > 0.0 {
            self.ticks_underwater.store((underwater - 1.0).max(0.0));
        }
    }

    fn init_data_tracker(&self) {
        self.send_wobble_metadata();
        self.vehicle.entity.send_meta_data(
            &[Metadata::new(
                pumpkin_data::tracked_data::boat::ID_TYPE,
                VarInt(self.boat_type.load(Ordering::Relaxed) as i32),
            )],
            None,
        );
    }

    fn write_custom_nbt(&self, nbt: &mut NbtCompound) {
        let (name, _, _) = BOAT_TYPES[self.boat_type.load(Ordering::Relaxed)];
        nbt.put_string("Type", name.to_string());
    }

    fn read_custom_nbt(&self, nbt: &NbtCompound) {
        if let Some(name) = nbt.get_string("Type")
            && let Some(index) = BOAT_TYPES.iter().position(|(n, _, _)| *n == name)
        {
            self.set_boat_type(index);
        }
    }

    fn can_hit(&self) -> bool {
        self.vehicle.entity.is_alive()
    }

    fn is_collidable(&self, _entity: Option<Box<dyn EntityBase>>) -> bool {
        true
    }

    fn damage_with_context(
        &self,
        _caller: &dyn EntityBase,
        amount: f32,
        _damage_type: DamageType,
        _position: Option<Vector3<f64>>,
        source: Option<&dyn EntityBase>,
        _cause: Option<&dyn EntityBase>,
    ) -> bool {
        self.vehicle.damage_with_context(amount, source)
    }

    fn interact(&self, player: &Arc<Player>, _item_stack: &mut ItemStack) -> bool {
        if player.get_entity().is_sneaking() {
            return false;
        }

        if self.ticks_underwater.load() >= 60.0 {
            return false;
        }

        if self
            .vehicle
            .entity
            .passengers
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .len()
            >= 2
        {
            return false;
        }

        if player.get_entity().has_vehicle() {
            return false;
        }

        let world = self.vehicle.entity.world.load();
        let Some(vehicle) = world.get_entity_by_id(self.vehicle.entity.entity_id) else {
            return false;
        };

        let Some(passenger) = world.get_player_by_id(player.entity_id()) else {
            return false;
        };

        self.vehicle
            .entity
            .add_passenger(vehicle, passenger as Arc<dyn EntityBase>);

        true
    }

    fn set_paddle_state(&self, left: bool, right: bool) {
        self.set_paddles(left, right);
    }
    fn cast_any(&self) -> &dyn std::any::Any {
        self
    }

    fn is_pushable(&self) -> bool {
        true
    }
}
