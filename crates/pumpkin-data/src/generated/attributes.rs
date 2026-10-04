/* This file is generated. Do not edit manually. */
use std::hash::Hash;
#[derive(Clone, Debug)]
pub struct Attributes {
    pub id: u8,
    pub default_value: f64,
    pub name: &'static str,
}
impl PartialEq for Attributes {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}
impl Eq for Attributes {}
impl Hash for Attributes {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}
impl Attributes {
    pub const GENERIC_ARMOR: Self = Self {
        id: 0,
        default_value: 0f64,
        name: "minecraft:generic.armor",
    };
    pub const GENERIC_ARMOR_TOUGHNESS: Self = Self {
        id: 1,
        default_value: 0f64,
        name: "minecraft:generic.armor_toughness",
    };
    pub const GENERIC_ATTACK_DAMAGE: Self = Self {
        id: 2,
        default_value: 2f64,
        name: "minecraft:generic.attack_damage",
    };
    pub const GENERIC_ATTACK_KNOCKBACK: Self = Self {
        id: 3,
        default_value: 0f64,
        name: "minecraft:generic.attack_knockback",
    };
    pub const GENERIC_ATTACK_SPEED: Self = Self {
        id: 4,
        default_value: 4f64,
        name: "minecraft:generic.attack_speed",
    };
    pub const PLAYER_BLOCK_BREAK_SPEED: Self = Self {
        id: 5,
        default_value: 1f64,
        name: "minecraft:player.block_break_speed",
    };
    pub const PLAYER_BLOCK_INTERACTION_RANGE: Self = Self {
        id: 6,
        default_value: 4.5f64,
        name: "minecraft:player.block_interaction_range",
    };
    pub const GENERIC_BURNING_TIME: Self = Self {
        id: 7,
        default_value: 1f64,
        name: "minecraft:generic.burning_time",
    };
    pub const GENERIC_EXPLOSION_KNOCKBACK_RESISTANCE: Self = Self {
        id: 8,
        default_value: 0f64,
        name: "minecraft:generic.explosion_knockback_resistance",
    };
    pub const PLAYER_ENTITY_INTERACTION_RANGE: Self = Self {
        id: 9,
        default_value: 3f64,
        name: "minecraft:player.entity_interaction_range",
    };
    pub const GENERIC_FALL_DAMAGE_MULTIPLIER: Self = Self {
        id: 10,
        default_value: 1f64,
        name: "minecraft:generic.fall_damage_multiplier",
    };
    pub const GENERIC_FLYING_SPEED: Self = Self {
        id: 11,
        default_value: 0.4f64,
        name: "minecraft:generic.flying_speed",
    };
    pub const GENERIC_FOLLOW_RANGE: Self = Self {
        id: 12,
        default_value: 32f64,
        name: "minecraft:generic.follow_range",
    };
    pub const GENERIC_GRAVITY: Self = Self {
        id: 13,
        default_value: 0.08f64,
        name: "minecraft:generic.gravity",
    };
    pub const GENERIC_JUMP_STRENGTH: Self = Self {
        id: 14,
        default_value: 0.41999998688697815f64,
        name: "minecraft:generic.jump_strength",
    };
    pub const GENERIC_KNOCKBACK_RESISTANCE: Self = Self {
        id: 15,
        default_value: 0f64,
        name: "minecraft:generic.knockback_resistance",
    };
    pub const GENERIC_LUCK: Self = Self {
        id: 16,
        default_value: 0f64,
        name: "minecraft:generic.luck",
    };
    pub const GENERIC_MAX_ABSORPTION: Self = Self {
        id: 17,
        default_value: 0f64,
        name: "minecraft:generic.max_absorption",
    };
    pub const GENERIC_MAX_HEALTH: Self = Self {
        id: 18,
        default_value: 20f64,
        name: "minecraft:generic.max_health",
    };
    pub const PLAYER_MINING_EFFICIENCY: Self = Self {
        id: 19,
        default_value: 0f64,
        name: "minecraft:player.mining_efficiency",
    };
    pub const GENERIC_MOVEMENT_EFFICIENCY: Self = Self {
        id: 20,
        default_value: 0f64,
        name: "minecraft:generic.movement_efficiency",
    };
    pub const GENERIC_MOVEMENT_SPEED: Self = Self {
        id: 21,
        default_value: 0.7f64,
        name: "minecraft:generic.movement_speed",
    };
    pub const GENERIC_OXYGEN_BONUS: Self = Self {
        id: 22,
        default_value: 0f64,
        name: "minecraft:generic.oxygen_bonus",
    };
    pub const GENERIC_SAFE_FALL_DISTANCE: Self = Self {
        id: 23,
        default_value: 3f64,
        name: "minecraft:generic.safe_fall_distance",
    };
    pub const GENERIC_SCALE: Self = Self {
        id: 24,
        default_value: 1f64,
        name: "minecraft:generic.scale",
    };
    pub const PLAYER_SNEAKING_SPEED: Self = Self {
        id: 25,
        default_value: 0.3f64,
        name: "minecraft:player.sneaking_speed",
    };
    pub const ZOMBIE_SPAWN_REINFORCEMENTS: Self = Self {
        id: 26,
        default_value: 0f64,
        name: "minecraft:zombie.spawn_reinforcements",
    };
    pub const GENERIC_STEP_HEIGHT: Self = Self {
        id: 27,
        default_value: 0.6f64,
        name: "minecraft:generic.step_height",
    };
    pub const PLAYER_SUBMERGED_MINING_SPEED: Self = Self {
        id: 28,
        default_value: 0.2f64,
        name: "minecraft:player.submerged_mining_speed",
    };
    pub const PLAYER_SWEEPING_DAMAGE_RATIO: Self = Self {
        id: 29,
        default_value: 0f64,
        name: "minecraft:player.sweeping_damage_ratio",
    };
    pub const GENERIC_WATER_MOVEMENT_EFFICIENCY: Self = Self {
        id: 30,
        default_value: 0f64,
        name: "minecraft:generic.water_movement_efficiency",
    };
    pub const ALL: &'static [Self] = &[
        Self::GENERIC_ARMOR,
        Self::GENERIC_ARMOR_TOUGHNESS,
        Self::GENERIC_ATTACK_DAMAGE,
        Self::GENERIC_ATTACK_KNOCKBACK,
        Self::GENERIC_ATTACK_SPEED,
        Self::PLAYER_BLOCK_BREAK_SPEED,
        Self::PLAYER_BLOCK_INTERACTION_RANGE,
        Self::GENERIC_BURNING_TIME,
        Self::GENERIC_EXPLOSION_KNOCKBACK_RESISTANCE,
        Self::PLAYER_ENTITY_INTERACTION_RANGE,
        Self::GENERIC_FALL_DAMAGE_MULTIPLIER,
        Self::GENERIC_FLYING_SPEED,
        Self::GENERIC_FOLLOW_RANGE,
        Self::GENERIC_GRAVITY,
        Self::GENERIC_JUMP_STRENGTH,
        Self::GENERIC_KNOCKBACK_RESISTANCE,
        Self::GENERIC_LUCK,
        Self::GENERIC_MAX_ABSORPTION,
        Self::GENERIC_MAX_HEALTH,
        Self::PLAYER_MINING_EFFICIENCY,
        Self::GENERIC_MOVEMENT_EFFICIENCY,
        Self::GENERIC_MOVEMENT_SPEED,
        Self::GENERIC_OXYGEN_BONUS,
        Self::GENERIC_SAFE_FALL_DISTANCE,
        Self::GENERIC_SCALE,
        Self::PLAYER_SNEAKING_SPEED,
        Self::ZOMBIE_SPAWN_REINFORCEMENTS,
        Self::GENERIC_STEP_HEIGHT,
        Self::PLAYER_SUBMERGED_MINING_SPEED,
        Self::PLAYER_SWEEPING_DAMAGE_RATIO,
        Self::GENERIC_WATER_MOVEMENT_EFFICIENCY,
    ];
}
