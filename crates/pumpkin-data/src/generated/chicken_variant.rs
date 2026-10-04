/* This file is generated. Do not edit manually. */
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Default)]
#[repr(u8)]
pub enum ChickenVariant {
    Cold = 0u8,
    #[default]
    Temperate = 1u8,
    Warm = 2u8,
}
impl ChickenVariant {
    pub const ALL: &'static [Self] = &[Self::Cold, Self::Temperate, Self::Warm];
    #[doc = "Returns the variant from a resource name (bare or namespaced)."]
    #[must_use]
    pub fn from_name(name: &str) -> Option<Self> {
        match name {
            "minecraft:cold" | "cold" => Some(Self::Cold),
            "minecraft:temperate" | "temperate" => Some(Self::Temperate),
            "minecraft:warm" | "warm" => Some(Self::Warm),
            _ => None,
        }
    }
    #[doc = "Returns the numeric ID of the variant in the synced registry."]
    #[must_use]
    pub const fn id(&self) -> u8 {
        *self as u8
    }
    #[must_use]
    pub const fn from_id(id: u8) -> Option<Self> {
        match id {
            0u8 => Some(Self::Cold),
            1u8 => Some(Self::Temperate),
            2u8 => Some(Self::Warm),
            _ => None,
        }
    }
    #[doc = "Returns the bare string name of the variant."]
    #[must_use]
    pub const fn to_name(&self) -> &'static str {
        match self {
            Self::Cold => "cold",
            Self::Temperate => "temperate",
            Self::Warm => "warm",
        }
    }
    #[must_use]
    pub const fn asset_id(&self) -> &'static str {
        match self {
            Self::Cold => "minecraft:entity/chicken/chicken_cold",
            Self::Temperate => "minecraft:entity/chicken/chicken_temperate",
            Self::Warm => "minecraft:entity/chicken/chicken_warm",
        }
    }
    #[must_use]
    pub const fn baby_asset_id(&self) -> Option<&'static str> {
        match self {
            Self::Cold => Some("minecraft:entity/chicken/chicken_cold_baby"),
            Self::Temperate => Some("minecraft:entity/chicken/chicken_temperate_baby"),
            Self::Warm => Some("minecraft:entity/chicken/chicken_warm_baby"),
        }
    }
    #[must_use]
    pub const fn model(&self) -> Option<&'static str> {
        match self {
            Self::Cold => Some("cold"),
            Self::Temperate => None,
            Self::Warm => None,
        }
    }
    #[must_use]
    pub const fn all() -> &'static [Self] {
        Self::ALL
    }
    #[doc = "The variant for a biome: always the default one, as 1.21.1 has no variants."]
    #[must_use]
    pub const fn select_for_biome(_biome_name: &str) -> Self {
        Self::Temperate
    }
}
