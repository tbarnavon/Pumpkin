/* This file is generated. Do not edit manually. */
use crate::data_component_impl::*;
#[derive(Copy, Clone, Hash, PartialEq, Eq)]
#[repr(u8)]
pub enum DataComponent {
    CustomData = 0u8,
    MaxStackSize = 1u8,
    MaxDamage = 2u8,
    Damage = 3u8,
    Unbreakable = 4u8,
    CustomName = 5u8,
    ItemName = 6u8,
    Lore = 7u8,
    Rarity = 8u8,
    Enchantments = 9u8,
    CanPlaceOn = 10u8,
    CanBreak = 11u8,
    AttributeModifiers = 12u8,
    CustomModelData = 13u8,
    HideAdditionalTooltip = 14u8,
    HideTooltip = 15u8,
    RepairCost = 16u8,
    CreativeSlotLock = 17u8,
    EnchantmentGlintOverride = 18u8,
    IntangibleProjectile = 19u8,
    Food = 20u8,
    FireResistant = 21u8,
    Tool = 22u8,
    StoredEnchantments = 23u8,
    DyedColor = 24u8,
    MapColor = 25u8,
    MapId = 26u8,
    MapDecorations = 27u8,
    MapPostProcessing = 28u8,
    ChargedProjectiles = 29u8,
    BundleContents = 30u8,
    PotionContents = 31u8,
    SuspiciousStewEffects = 32u8,
    WritableBookContent = 33u8,
    WrittenBookContent = 34u8,
    Trim = 35u8,
    DebugStickState = 36u8,
    EntityData = 37u8,
    BucketEntityData = 38u8,
    BlockEntityData = 39u8,
    Instrument = 40u8,
    OminousBottleAmplifier = 41u8,
    JukeboxPlayable = 42u8,
    Recipes = 43u8,
    LodestoneTracker = 44u8,
    FireworkExplosion = 45u8,
    Fireworks = 46u8,
    Profile = 47u8,
    NoteBlockSound = 48u8,
    BannerPatterns = 49u8,
    BaseColor = 50u8,
    PotDecorations = 51u8,
    Container = 52u8,
    BlockState = 53u8,
    Bees = 54u8,
    Lock = 55u8,
    ContainerLoot = 56u8,
}
impl DataComponent {
    #[must_use]
    pub const fn to_id(self) -> u8 {
        self as u8
    }
    #[must_use]
    #[allow(clippy::too_many_lines)]
    pub const fn try_from_id(id: u8) -> Option<Self> {
        match id {
            0u8 => Some(Self::CustomData),
            1u8 => Some(Self::MaxStackSize),
            2u8 => Some(Self::MaxDamage),
            3u8 => Some(Self::Damage),
            4u8 => Some(Self::Unbreakable),
            5u8 => Some(Self::CustomName),
            6u8 => Some(Self::ItemName),
            7u8 => Some(Self::Lore),
            8u8 => Some(Self::Rarity),
            9u8 => Some(Self::Enchantments),
            10u8 => Some(Self::CanPlaceOn),
            11u8 => Some(Self::CanBreak),
            12u8 => Some(Self::AttributeModifiers),
            13u8 => Some(Self::CustomModelData),
            14u8 => Some(Self::HideAdditionalTooltip),
            15u8 => Some(Self::HideTooltip),
            16u8 => Some(Self::RepairCost),
            17u8 => Some(Self::CreativeSlotLock),
            18u8 => Some(Self::EnchantmentGlintOverride),
            19u8 => Some(Self::IntangibleProjectile),
            20u8 => Some(Self::Food),
            21u8 => Some(Self::FireResistant),
            22u8 => Some(Self::Tool),
            23u8 => Some(Self::StoredEnchantments),
            24u8 => Some(Self::DyedColor),
            25u8 => Some(Self::MapColor),
            26u8 => Some(Self::MapId),
            27u8 => Some(Self::MapDecorations),
            28u8 => Some(Self::MapPostProcessing),
            29u8 => Some(Self::ChargedProjectiles),
            30u8 => Some(Self::BundleContents),
            31u8 => Some(Self::PotionContents),
            32u8 => Some(Self::SuspiciousStewEffects),
            33u8 => Some(Self::WritableBookContent),
            34u8 => Some(Self::WrittenBookContent),
            35u8 => Some(Self::Trim),
            36u8 => Some(Self::DebugStickState),
            37u8 => Some(Self::EntityData),
            38u8 => Some(Self::BucketEntityData),
            39u8 => Some(Self::BlockEntityData),
            40u8 => Some(Self::Instrument),
            41u8 => Some(Self::OminousBottleAmplifier),
            42u8 => Some(Self::JukeboxPlayable),
            43u8 => Some(Self::Recipes),
            44u8 => Some(Self::LodestoneTracker),
            45u8 => Some(Self::FireworkExplosion),
            46u8 => Some(Self::Fireworks),
            47u8 => Some(Self::Profile),
            48u8 => Some(Self::NoteBlockSound),
            49u8 => Some(Self::BannerPatterns),
            50u8 => Some(Self::BaseColor),
            51u8 => Some(Self::PotDecorations),
            52u8 => Some(Self::Container),
            53u8 => Some(Self::BlockState),
            54u8 => Some(Self::Bees),
            55u8 => Some(Self::Lock),
            56u8 => Some(Self::ContainerLoot),
            _ => None,
        }
    }
    #[must_use]
    #[allow(clippy::too_many_lines)]
    pub fn try_from_name(name: &str) -> Option<Self> {
        match name {
            "minecraft:custom_data" | "custom_data" => Some(Self::CustomData),
            "minecraft:max_stack_size" | "max_stack_size" => Some(Self::MaxStackSize),
            "minecraft:max_damage" | "max_damage" => Some(Self::MaxDamage),
            "minecraft:damage" | "damage" => Some(Self::Damage),
            "minecraft:unbreakable" | "unbreakable" => Some(Self::Unbreakable),
            "minecraft:custom_name" | "custom_name" => Some(Self::CustomName),
            "minecraft:item_name" | "item_name" => Some(Self::ItemName),
            "minecraft:lore" | "lore" => Some(Self::Lore),
            "minecraft:rarity" | "rarity" => Some(Self::Rarity),
            "minecraft:enchantments" | "enchantments" => Some(Self::Enchantments),
            "minecraft:can_place_on" | "can_place_on" => Some(Self::CanPlaceOn),
            "minecraft:can_break" | "can_break" => Some(Self::CanBreak),
            "minecraft:attribute_modifiers" | "attribute_modifiers" => {
                Some(Self::AttributeModifiers)
            }
            "minecraft:custom_model_data" | "custom_model_data" => Some(Self::CustomModelData),
            "minecraft:hide_additional_tooltip" | "hide_additional_tooltip" => {
                Some(Self::HideAdditionalTooltip)
            }
            "minecraft:hide_tooltip" | "hide_tooltip" => Some(Self::HideTooltip),
            "minecraft:repair_cost" | "repair_cost" => Some(Self::RepairCost),
            "minecraft:creative_slot_lock" | "creative_slot_lock" => Some(Self::CreativeSlotLock),
            "minecraft:enchantment_glint_override" | "enchantment_glint_override" => {
                Some(Self::EnchantmentGlintOverride)
            }
            "minecraft:intangible_projectile" | "intangible_projectile" => {
                Some(Self::IntangibleProjectile)
            }
            "minecraft:food" | "food" => Some(Self::Food),
            "minecraft:fire_resistant" | "fire_resistant" => Some(Self::FireResistant),
            "minecraft:tool" | "tool" => Some(Self::Tool),
            "minecraft:stored_enchantments" | "stored_enchantments" => {
                Some(Self::StoredEnchantments)
            }
            "minecraft:dyed_color" | "dyed_color" => Some(Self::DyedColor),
            "minecraft:map_color" | "map_color" => Some(Self::MapColor),
            "minecraft:map_id" | "map_id" => Some(Self::MapId),
            "minecraft:map_decorations" | "map_decorations" => Some(Self::MapDecorations),
            "minecraft:map_post_processing" | "map_post_processing" => {
                Some(Self::MapPostProcessing)
            }
            "minecraft:charged_projectiles" | "charged_projectiles" => {
                Some(Self::ChargedProjectiles)
            }
            "minecraft:bundle_contents" | "bundle_contents" => Some(Self::BundleContents),
            "minecraft:potion_contents" | "potion_contents" => Some(Self::PotionContents),
            "minecraft:suspicious_stew_effects" | "suspicious_stew_effects" => {
                Some(Self::SuspiciousStewEffects)
            }
            "minecraft:writable_book_content" | "writable_book_content" => {
                Some(Self::WritableBookContent)
            }
            "minecraft:written_book_content" | "written_book_content" => {
                Some(Self::WrittenBookContent)
            }
            "minecraft:trim" | "trim" => Some(Self::Trim),
            "minecraft:debug_stick_state" | "debug_stick_state" => Some(Self::DebugStickState),
            "minecraft:entity_data" | "entity_data" => Some(Self::EntityData),
            "minecraft:bucket_entity_data" | "bucket_entity_data" => Some(Self::BucketEntityData),
            "minecraft:block_entity_data" | "block_entity_data" => Some(Self::BlockEntityData),
            "minecraft:instrument" | "instrument" => Some(Self::Instrument),
            "minecraft:ominous_bottle_amplifier" | "ominous_bottle_amplifier" => {
                Some(Self::OminousBottleAmplifier)
            }
            "minecraft:jukebox_playable" | "jukebox_playable" => Some(Self::JukeboxPlayable),
            "minecraft:recipes" | "recipes" => Some(Self::Recipes),
            "minecraft:lodestone_tracker" | "lodestone_tracker" => Some(Self::LodestoneTracker),
            "minecraft:firework_explosion" | "firework_explosion" => Some(Self::FireworkExplosion),
            "minecraft:fireworks" | "fireworks" => Some(Self::Fireworks),
            "minecraft:profile" | "profile" => Some(Self::Profile),
            "minecraft:note_block_sound" | "note_block_sound" => Some(Self::NoteBlockSound),
            "minecraft:banner_patterns" | "banner_patterns" => Some(Self::BannerPatterns),
            "minecraft:base_color" | "base_color" => Some(Self::BaseColor),
            "minecraft:pot_decorations" | "pot_decorations" => Some(Self::PotDecorations),
            "minecraft:container" | "container" => Some(Self::Container),
            "minecraft:block_state" | "block_state" => Some(Self::BlockState),
            "minecraft:bees" | "bees" => Some(Self::Bees),
            "minecraft:lock" | "lock" => Some(Self::Lock),
            "minecraft:container_loot" | "container_loot" => Some(Self::ContainerLoot),
            _ => None,
        }
    }
    #[must_use]
    #[allow(clippy::too_many_lines)]
    pub const fn to_name(self) -> &'static str {
        match self {
            Self::CustomData => "minecraft:custom_data",
            Self::MaxStackSize => "minecraft:max_stack_size",
            Self::MaxDamage => "minecraft:max_damage",
            Self::Damage => "minecraft:damage",
            Self::Unbreakable => "minecraft:unbreakable",
            Self::CustomName => "minecraft:custom_name",
            Self::ItemName => "minecraft:item_name",
            Self::Lore => "minecraft:lore",
            Self::Rarity => "minecraft:rarity",
            Self::Enchantments => "minecraft:enchantments",
            Self::CanPlaceOn => "minecraft:can_place_on",
            Self::CanBreak => "minecraft:can_break",
            Self::AttributeModifiers => "minecraft:attribute_modifiers",
            Self::CustomModelData => "minecraft:custom_model_data",
            Self::HideAdditionalTooltip => "minecraft:hide_additional_tooltip",
            Self::HideTooltip => "minecraft:hide_tooltip",
            Self::RepairCost => "minecraft:repair_cost",
            Self::CreativeSlotLock => "minecraft:creative_slot_lock",
            Self::EnchantmentGlintOverride => "minecraft:enchantment_glint_override",
            Self::IntangibleProjectile => "minecraft:intangible_projectile",
            Self::Food => "minecraft:food",
            Self::FireResistant => "minecraft:fire_resistant",
            Self::Tool => "minecraft:tool",
            Self::StoredEnchantments => "minecraft:stored_enchantments",
            Self::DyedColor => "minecraft:dyed_color",
            Self::MapColor => "minecraft:map_color",
            Self::MapId => "minecraft:map_id",
            Self::MapDecorations => "minecraft:map_decorations",
            Self::MapPostProcessing => "minecraft:map_post_processing",
            Self::ChargedProjectiles => "minecraft:charged_projectiles",
            Self::BundleContents => "minecraft:bundle_contents",
            Self::PotionContents => "minecraft:potion_contents",
            Self::SuspiciousStewEffects => "minecraft:suspicious_stew_effects",
            Self::WritableBookContent => "minecraft:writable_book_content",
            Self::WrittenBookContent => "minecraft:written_book_content",
            Self::Trim => "minecraft:trim",
            Self::DebugStickState => "minecraft:debug_stick_state",
            Self::EntityData => "minecraft:entity_data",
            Self::BucketEntityData => "minecraft:bucket_entity_data",
            Self::BlockEntityData => "minecraft:block_entity_data",
            Self::Instrument => "minecraft:instrument",
            Self::OminousBottleAmplifier => "minecraft:ominous_bottle_amplifier",
            Self::JukeboxPlayable => "minecraft:jukebox_playable",
            Self::Recipes => "minecraft:recipes",
            Self::LodestoneTracker => "minecraft:lodestone_tracker",
            Self::FireworkExplosion => "minecraft:firework_explosion",
            Self::Fireworks => "minecraft:fireworks",
            Self::Profile => "minecraft:profile",
            Self::NoteBlockSound => "minecraft:note_block_sound",
            Self::BannerPatterns => "minecraft:banner_patterns",
            Self::BaseColor => "minecraft:base_color",
            Self::PotDecorations => "minecraft:pot_decorations",
            Self::Container => "minecraft:container",
            Self::BlockState => "minecraft:block_state",
            Self::Bees => "minecraft:bees",
            Self::Lock => "minecraft:lock",
            Self::ContainerLoot => "minecraft:container_loot",
        }
    }
}
