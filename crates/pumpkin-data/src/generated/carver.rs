/* This file is generated. Do not edit manually. */
use pumpkin_util::math::float_provider::{
    ClampedNormalFloatProvider, ConstantFloatProvider, FloatProvider, NormalFloatProvider,
    TrapezoidFloatProvider, UniformFloatProvider,
};
use pumpkin_util::math::int_provider::{
    BiasedToBottomIntProvider, ConstantIntProvider, IntProvider, NormalIntProvider,
    UniformIntProvider, VeryBiasedToBottomIntProvider,
};
use pumpkin_util::y_offset::{AboveBottom, Absolute, BelowTop, YOffset};
#[derive(Clone, Debug)]
pub enum HeightProvider {
    Uniform(UniformHeightProvider),
    Trapezoid(TrapezoidHeightProvider),
    VeryBiasedToBottom(VeryBiasedToBottomHeightProvider),
}
#[derive(Clone, Debug)]
pub struct UniformHeightProvider {
    pub min_inclusive: YOffset,
    pub max_inclusive: YOffset,
}
#[derive(Clone, Debug)]
pub struct TrapezoidHeightProvider {
    pub min_inclusive: YOffset,
    pub max_inclusive: YOffset,
    pub plateau: Option<i32>,
}
#[derive(Clone, Debug)]
pub struct VeryBiasedToBottomHeightProvider {
    pub min_inclusive: YOffset,
    pub max_inclusive: YOffset,
    pub inner: Option<std::num::NonZero<u32>>,
}
#[derive(Clone, Debug)]
pub struct CaveCarverConfig {
    pub count: IntProvider,
    pub horizontal_radius_multiplier: FloatProvider,
    pub vertical_radius_multiplier: FloatProvider,
    pub floor_level: FloatProvider,
    pub room_vertical_radius_multiplier: FloatProvider,
    pub start_vertical_radius_multiplier: FloatProvider,
    pub thickness: FloatProvider,
    pub weird_thickness_bias: bool,
}
impl CaveCarverConfig {
    #[must_use]
    pub const fn default() -> Self {
        Self {
            count: IntProvider::Constant(1),
            horizontal_radius_multiplier: FloatProvider::Constant(1.0),
            vertical_radius_multiplier: FloatProvider::Constant(1.0),
            floor_level: FloatProvider::Constant(-0.7),
            room_vertical_radius_multiplier: FloatProvider::Constant(0.5),
            start_vertical_radius_multiplier: FloatProvider::Constant(1.0),
            thickness: FloatProvider::Constant(1.0),
            weird_thickness_bias: false,
        }
    }
}
#[derive(Clone, Debug)]
pub struct CanyonShapeConfig {
    pub distance_factor: FloatProvider,
    pub thickness: FloatProvider,
    pub width_smoothness: i32,
    pub horizontal_radius_factor: FloatProvider,
    pub vertical_radius_default_factor: f32,
    pub vertical_radius_center_factor: f32,
    pub y_scale: FloatProvider,
}
#[derive(Clone, Debug)]
pub struct CanyonCarverConfig {
    pub vertical_rotation: FloatProvider,
    pub shape: CanyonShapeConfig,
}
#[derive(Clone, Debug)]
pub enum CarverAdditionalConfig {
    Cave(CaveCarverConfig),
    Canyon(CanyonCarverConfig),
}
#[derive(Clone, Debug)]
pub struct CarverConfig {
    pub probability: f32,
    pub y: HeightProvider,
    pub additional: CarverAdditionalConfig,
}
use super::*;
pub const CANYON: CarverConfig = CarverConfig {
    probability: 0f32,
    y: HeightProvider::Uniform(UniformHeightProvider {
        min_inclusive: YOffset::Absolute(Absolute { absolute: 0 }),
        max_inclusive: YOffset::Absolute(Absolute { absolute: 0 }),
    }),
    additional: CarverAdditionalConfig::Canyon(CanyonCarverConfig {
        vertical_rotation: FloatProvider::Constant(0.0),
        shape: CanyonShapeConfig {
            distance_factor: FloatProvider::Constant(0.0),
            thickness: FloatProvider::Constant(0.0),
            width_smoothness: 0i32,
            horizontal_radius_factor: FloatProvider::Constant(0.0),
            vertical_radius_default_factor: 0f32,
            vertical_radius_center_factor: 0f32,
            y_scale: FloatProvider::Constant(0.0),
        },
    }),
};
pub const CAVE: CarverConfig = CarverConfig {
    probability: 0f32,
    y: HeightProvider::Uniform(UniformHeightProvider {
        min_inclusive: YOffset::Absolute(Absolute { absolute: 0 }),
        max_inclusive: YOffset::Absolute(Absolute { absolute: 0 }),
    }),
    additional: CarverAdditionalConfig::Cave(CaveCarverConfig {
        count: IntProvider::Constant(0),
        horizontal_radius_multiplier: FloatProvider::Constant(0.0),
        vertical_radius_multiplier: FloatProvider::Constant(0.0),
        floor_level: FloatProvider::Constant(0.0),
        room_vertical_radius_multiplier: FloatProvider::Constant(0.0),
        start_vertical_radius_multiplier: FloatProvider::Constant(1.0),
        thickness: FloatProvider::Constant(0.0),
        weird_thickness_bias: false,
    }),
};
pub const CAVE_EXTRA_UNDERGROUND: CarverConfig = CarverConfig {
    probability: 0f32,
    y: HeightProvider::Uniform(UniformHeightProvider {
        min_inclusive: YOffset::Absolute(Absolute { absolute: 0 }),
        max_inclusive: YOffset::Absolute(Absolute { absolute: 0 }),
    }),
    additional: CarverAdditionalConfig::Cave(CaveCarverConfig {
        count: IntProvider::Constant(0),
        horizontal_radius_multiplier: FloatProvider::Constant(0.0),
        vertical_radius_multiplier: FloatProvider::Constant(0.0),
        floor_level: FloatProvider::Constant(0.0),
        room_vertical_radius_multiplier: FloatProvider::Constant(0.0),
        start_vertical_radius_multiplier: FloatProvider::Constant(1.0),
        thickness: FloatProvider::Constant(0.0),
        weird_thickness_bias: false,
    }),
};
pub const NETHER_CAVE: CarverConfig = CarverConfig {
    probability: 0f32,
    y: HeightProvider::Uniform(UniformHeightProvider {
        min_inclusive: YOffset::Absolute(Absolute { absolute: 0 }),
        max_inclusive: YOffset::Absolute(Absolute { absolute: 0 }),
    }),
    additional: CarverAdditionalConfig::Cave(CaveCarverConfig::default()),
};
