/* This file is generated. Do not edit manually. */
use serde::Serialize;
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct VillagerTradeItem {
    pub item: &'static crate::item::Item,
    pub count: i32,
}
#[derive(Clone, Copy, PartialEq)]
pub struct VillagerTrade {
    pub wants: VillagerTradeItem,
    pub wants_b: Option<VillagerTradeItem>,
    pub gives: VillagerTradeItem,
    pub max_uses: i32,
    pub xp: i32,
    pub price_multiplier: f32,
    pub modifier: VillagerTradeModifier,
    pub allowed_types: &'static [VillagerType],
}
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum VillagerTradeModifier {
    None,
    EnchantRandomly,
    EnchantWithLevels { min: i32, max: i32 },
    ExplorationMap { destination: &'static str },
    RandomDyes,
    RandomPotion,
    SuspiciousStew,
    Potion(&'static str),
}
#[derive(Clone, Copy, PartialEq)]
pub struct VillagerTradeSet {
    pub trades: &'static [VillagerTrade],
    pub amount: i32,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
#[repr(i32)]
pub enum VillagerProfession {
    None,
    Armorer,
    Butcher,
    Cartographer,
    Cleric,
    Farmer,
    Fisherman,
    Fletcher,
    Leatherworker,
    Librarian,
    Mason,
    Nitwit,
    Shepherd,
    Toolsmith,
    Weaponsmith,
}
impl VillagerProfession {
    #[must_use]
    pub const fn from_i32(id: i32) -> Option<Self> {
        match id {
            0i32 => Some(Self::None),
            1i32 => Some(Self::Armorer),
            2i32 => Some(Self::Butcher),
            3i32 => Some(Self::Cartographer),
            4i32 => Some(Self::Cleric),
            5i32 => Some(Self::Farmer),
            6i32 => Some(Self::Fisherman),
            7i32 => Some(Self::Fletcher),
            8i32 => Some(Self::Leatherworker),
            9i32 => Some(Self::Librarian),
            10i32 => Some(Self::Mason),
            11i32 => Some(Self::Nitwit),
            12i32 => Some(Self::Shepherd),
            13i32 => Some(Self::Toolsmith),
            14i32 => Some(Self::Weaponsmith),
            _ => None,
        }
    }
    #[must_use]
    #[allow(clippy::match_same_arms)]
    pub const fn work_sound(&self) -> Option<crate::sound::Sound> {
        match self {
            Self::None => None,
            Self::Armorer => Some(crate::sound::Sound::EntityVillagerWorkArmorer),
            Self::Butcher => Some(crate::sound::Sound::EntityVillagerWorkButcher),
            Self::Cartographer => Some(crate::sound::Sound::EntityVillagerWorkCartographer),
            Self::Cleric => Some(crate::sound::Sound::EntityVillagerWorkCleric),
            Self::Farmer => Some(crate::sound::Sound::EntityVillagerWorkFarmer),
            Self::Fisherman => Some(crate::sound::Sound::EntityVillagerWorkFisherman),
            Self::Fletcher => Some(crate::sound::Sound::EntityVillagerWorkFletcher),
            Self::Leatherworker => Some(crate::sound::Sound::EntityVillagerWorkLeatherworker),
            Self::Librarian => Some(crate::sound::Sound::EntityVillagerWorkLibrarian),
            Self::Mason => Some(crate::sound::Sound::EntityVillagerWorkMason),
            Self::Nitwit => None,
            Self::Shepherd => Some(crate::sound::Sound::EntityVillagerWorkShepherd),
            Self::Toolsmith => Some(crate::sound::Sound::EntityVillagerWorkToolsmith),
            Self::Weaponsmith => Some(crate::sound::Sound::EntityVillagerWorkWeaponsmith),
        }
    }
    #[must_use]
    #[allow(clippy::match_same_arms)]
    pub const fn requested_items(&self) -> &'static [&'static crate::item::Item] {
        match self {
            Self::None => &[],
            Self::Armorer => &[],
            Self::Butcher => &[],
            Self::Cartographer => &[],
            Self::Cleric => &[],
            Self::Farmer => &[
                &crate::item::Item::WHEAT,
                &crate::item::Item::WHEAT_SEEDS,
                &crate::item::Item::BEETROOT_SEEDS,
                &crate::item::Item::BONE_MEAL,
            ],
            Self::Fisherman => &[],
            Self::Fletcher => &[],
            Self::Leatherworker => &[],
            Self::Librarian => &[],
            Self::Mason => &[],
            Self::Nitwit => &[],
            Self::Shepherd => &[],
            Self::Toolsmith => &[],
            Self::Weaponsmith => &[],
        }
    }
    #[must_use]
    pub const fn translation_key(&self) -> &'static str {
        match self {
            Self::None => "entity.minecraft.villager.none",
            Self::Armorer => "entity.minecraft.villager.armorer",
            Self::Butcher => "entity.minecraft.villager.butcher",
            Self::Cartographer => "entity.minecraft.villager.cartographer",
            Self::Cleric => "entity.minecraft.villager.cleric",
            Self::Farmer => "entity.minecraft.villager.farmer",
            Self::Fisherman => "entity.minecraft.villager.fisherman",
            Self::Fletcher => "entity.minecraft.villager.fletcher",
            Self::Leatherworker => "entity.minecraft.villager.leatherworker",
            Self::Librarian => "entity.minecraft.villager.librarian",
            Self::Mason => "entity.minecraft.villager.mason",
            Self::Nitwit => "entity.minecraft.villager.nitwit",
            Self::Shepherd => "entity.minecraft.villager.shepherd",
            Self::Toolsmith => "entity.minecraft.villager.toolsmith",
            Self::Weaponsmith => "entity.minecraft.villager.weaponsmith",
        }
    }
    #[must_use]
    pub const fn to_name(&self) -> &'static str {
        match self {
            Self::None => "none",
            Self::Armorer => "armorer",
            Self::Butcher => "butcher",
            Self::Cartographer => "cartographer",
            Self::Cleric => "cleric",
            Self::Farmer => "farmer",
            Self::Fisherman => "fisherman",
            Self::Fletcher => "fletcher",
            Self::Leatherworker => "leatherworker",
            Self::Librarian => "librarian",
            Self::Mason => "mason",
            Self::Nitwit => "nitwit",
            Self::Shepherd => "shepherd",
            Self::Toolsmith => "toolsmith",
            Self::Weaponsmith => "weaponsmith",
        }
    }
    #[must_use]
    pub fn from_name(name: &str) -> Option<Self> {
        let clean = match name.strip_prefix("minecraft:") {
            Some(stripped) => stripped,
            None => name,
        };
        match clean {
            "none" => Some(Self::None),
            "armorer" => Some(Self::Armorer),
            "butcher" => Some(Self::Butcher),
            "cartographer" => Some(Self::Cartographer),
            "cleric" => Some(Self::Cleric),
            "farmer" => Some(Self::Farmer),
            "fisherman" => Some(Self::Fisherman),
            "fletcher" => Some(Self::Fletcher),
            "leatherworker" => Some(Self::Leatherworker),
            "librarian" => Some(Self::Librarian),
            "mason" => Some(Self::Mason),
            "nitwit" => Some(Self::Nitwit),
            "shepherd" => Some(Self::Shepherd),
            "toolsmith" => Some(Self::Toolsmith),
            "weaponsmith" => Some(Self::Weaponsmith),
            _ => None,
        }
    }
    #[must_use]
    #[allow(clippy::too_many_lines, clippy::match_same_arms)]
    pub const fn trade_set(&self, level: i32) -> Option<VillagerTradeSet> {
        match self {
            Self::None => None,
            Self::Armorer => None,
            Self::Butcher => None,
            Self::Cartographer => None,
            Self::Cleric => None,
            Self::Farmer => None,
            Self::Fisherman => None,
            Self::Fletcher => None,
            Self::Leatherworker => None,
            Self::Librarian => None,
            Self::Mason => None,
            Self::Nitwit => None,
            Self::Shepherd => None,
            Self::Toolsmith => None,
            Self::Weaponsmith => None,
        }
    }
}
impl TryFrom<i32> for VillagerProfession {
    type Error = ();
    fn try_from(value: i32) -> Result<Self, Self::Error> {
        Self::from_i32(value).ok_or(())
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
#[repr(i32)]
pub enum VillagerType {
    Desert,
    Jungle,
    Plains,
    Savanna,
    Snow,
    Swamp,
    Taiga,
}
impl VillagerType {
    #[must_use]
    pub const fn from_i32(id: i32) -> Option<Self> {
        match id {
            0i32 => Some(Self::Desert),
            1i32 => Some(Self::Jungle),
            2i32 => Some(Self::Plains),
            3i32 => Some(Self::Savanna),
            4i32 => Some(Self::Snow),
            5i32 => Some(Self::Swamp),
            6i32 => Some(Self::Taiga),
            _ => None,
        }
    }
    #[must_use]
    pub const fn to_name(&self) -> &'static str {
        match self {
            Self::Desert => "desert",
            Self::Jungle => "jungle",
            Self::Plains => "plains",
            Self::Savanna => "savanna",
            Self::Snow => "snow",
            Self::Swamp => "swamp",
            Self::Taiga => "taiga",
        }
    }
    #[must_use]
    pub fn from_name(name: &str) -> Option<Self> {
        let clean = match name.strip_prefix("minecraft:") {
            Some(stripped) => stripped,
            None => name,
        };
        match clean {
            "desert" => Some(Self::Desert),
            "jungle" => Some(Self::Jungle),
            "plains" => Some(Self::Plains),
            "savanna" => Some(Self::Savanna),
            "snow" => Some(Self::Snow),
            "swamp" => Some(Self::Swamp),
            "taiga" => Some(Self::Taiga),
            _ => None,
        }
    }
}
impl TryFrom<i32> for VillagerType {
    type Error = ();
    fn try_from(value: i32) -> Result<Self, Self::Error> {
        Self::from_i32(value).ok_or(())
    }
}
