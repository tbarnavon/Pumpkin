//! Item stacks and their components in the 1.20.5 to 1.21.1 network layouts
//! (`DataComponents` and the component classes' `STREAM_CODEC`s in 1.21.1).
//!
//! Later versions dropped the `show_in_tooltip` flags, reshaped food, tools, custom model data
//! and potion contents, and added components 1.21.1 clients don't know. Components without a
//! 1.21.1 network codec are sent as network NBT, like vanilla's default
//! (`ByteBufCodecs.fromCodecWithRegistries`). A component the server can't express in 1.21.1's
//! form is left out of the patch rather than sent wrong.

use std::borrow::Cow;

use pumpkin_data::banner_pattern::BannerPattern;
use pumpkin_data::data_component::DataComponent;
use pumpkin_data::data_component_impl::{
    AttributeModifiersImpl, BannerPatternLayer, BannerPatternsImpl, BeesImpl, BlockEntityDataImpl,
    BundleContentsImpl, ChargedProjectilesImpl, ContainerImpl, CustomModelDataImpl,
    DataComponentImpl, DyedColorImpl, EnchantmentsImpl, EntityDataImpl, FoodImpl, InstrumentImpl,
    ItemNameImpl, JukeboxPlayableImpl, LoreImpl, PotDecorationsImpl, PotionContentsImpl,
    ProfileImpl, ProfileProperty, StoredEnchantmentsImpl, ToolImpl, ToolRule, TrimImpl,
    UnbreakableImpl, WritableBookContentImpl, WrittenBookContentImpl, get, read_data,
};
use pumpkin_data::dye_color::DyeColor;
use pumpkin_data::enchantment::AttributeModifierSlot;
use pumpkin_data::item::Item;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_data::trim_material::TrimMaterial;
use pumpkin_data::trim_pattern::TrimPattern;
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_nbt::tag::NbtTag;
use pumpkin_util::text::TextComponent;
use pumpkin_util::version::JavaMinecraftVersion;

use super::data_component::{
    DataComponentCodec, deserialize, deserialize_idset, deserialize_status_effects, serialize,
    serialize_idset, serialize_status_effects,
};
use super::item_stack_seralizer::{
    read_unknown_component, write_unknown_added, write_unknown_removed,
};
use crate::codec::var_int::VarInt;
use crate::ser::{NetworkReadExt, NetworkWriteExt, ReadingError, WritingError};

const VERSION: JavaMinecraftVersion = JavaMinecraftVersion::V_1_21;
const MAX_LIST: i32 = 256;

/// Writes an item stack (`ItemStack.OPTIONAL_STREAM_CODEC`) of the item with network id
/// `item_id`.
pub fn write_stack(
    stack: &ItemStack,
    item_id: u16,
    write: &mut impl NetworkWriteExt,
) -> Result<(), WritingError> {
    if stack.is_empty() {
        return write.write_var_int(&VarInt(0));
    }
    let (added, removed) = encode_patch(stack)?;
    let unknown_added = stack
        .unknown_patch
        .iter()
        .filter(|(_, d)| d.is_some())
        .count();
    let unknown_removed = stack.unknown_patch.len() - unknown_added;

    write.write_var_int(&VarInt::from(stack.item_count))?;
    write.write_var_int(&VarInt::from(item_id))?;
    write.write_var_int(&VarInt((added.len() + unknown_added) as i32))?;
    write.write_var_int(&VarInt((removed.len() + unknown_removed) as i32))?;
    for (id, bytes) in &added {
        write.write_var_int(&VarInt(i32::from(id.to_id())))?;
        write.write_slice(bytes)?;
    }
    write_unknown_added(stack, false, write)?;
    for id in &removed {
        write.write_var_int(&VarInt(i32::from(id.to_id())))?;
    }
    write_unknown_removed(stack, write)
}

/// Writes the added components as a `DataComponentPredicate` (an item cost's components).
pub fn write_component_list(
    stack: &ItemStack,
    write: &mut impl NetworkWriteExt,
) -> Result<(), WritingError> {
    let (added, _) = encode_patch(stack)?;
    write.write_var_int(&VarInt(added.len() as i32))?;
    for (id, bytes) in &added {
        write.write_var_int(&VarInt(i32::from(id.to_id())))?;
        write.write_slice(bytes)?;
    }
    Ok(())
}

type EncodedPatch = (Vec<(DataComponent, Vec<u8>)>, Vec<DataComponent>);

/// Encodes the patch's components 1.21.1 knows: the added ones with their values, and the
/// removed ones.
fn encode_patch(stack: &ItemStack) -> Result<EncodedPatch, WritingError> {
    let mut added = Vec::new();
    let mut removed = Vec::new();
    for (id, data) in &stack.patch {
        if !id.is_networked() {
            continue;
        }
        match data {
            Some(data) => {
                let mut bytes = Vec::new();
                if write_component(*id, data.as_ref(), &mut bytes)? {
                    added.push((*id, bytes));
                }
            }
            None => removed.push(*id),
        }
    }
    Ok((added, removed))
}

/// Reads an item stack (`ItemStack.OPTIONAL_STREAM_CODEC`). Components the server can't keep
/// are read and dropped.
pub fn read_stack(read: &mut impl NetworkReadExt) -> Result<ItemStack, ReadingError> {
    let count = read.get_var_int()?.0;
    if count <= 0 {
        return Ok(ItemStack::EMPTY.clone());
    }
    let item_id = u16::try_from(read.get_var_int()?.0)
        .map_err(|_| ReadingError::Message("Invalid item id".into()))?;
    let to_add = read.get_var_int()?.0;
    let to_remove = read.get_var_int()?.0;
    if !(0..=MAX_LIST).contains(&to_add) || !(0..=MAX_LIST).contains(&to_remove) {
        return Err(ReadingError::Message("Bad component count".into()));
    }
    let mut patch = Vec::new();
    let mut unknown_patch = Vec::new();
    for _ in 0..to_add {
        let raw = read.get_var_int()?.0;
        let raw = u16::try_from(raw)
            .map_err(|_| ReadingError::Message(format!("Invalid component id {raw}")))?;
        if let Some(value) = read_unknown_component(read, raw)? {
            unknown_patch.push((raw, Some(value)));
            continue;
        }
        let id = u8::try_from(raw)
            .ok()
            .and_then(DataComponent::try_from_id)
            .filter(|id| id.is_networked())
            .ok_or_else(|| ReadingError::Message(format!("Unknown component id {raw}")))?;
        if let Some(value) = read_component(id, read)? {
            patch.push((id, Some(value)));
        }
    }
    for _ in 0..to_remove {
        let raw = read.get_var_int()?.0;
        let raw = u16::try_from(raw)
            .map_err(|_| ReadingError::Message(format!("Invalid component id {raw}")))?;
        if pumpkin_data::item_stack::unknown_component_name(raw).is_some() {
            unknown_patch.push((raw, None));
        } else if let Some(id) = u8::try_from(raw).ok().and_then(DataComponent::try_from_id) {
            patch.push((id, None));
        }
    }
    let mut stack = ItemStack::new_with_component(
        count.min(i32::from(u8::MAX)) as u8,
        Item::from_id(item_id).unwrap_or(&Item::AIR),
        patch,
    );
    stack.unknown_patch = unknown_patch;
    Ok(stack)
}

fn write_list<T>(
    write: &mut impl NetworkWriteExt,
    items: &[T],
    mut each: impl FnMut(&mut Vec<u8>, &T) -> Result<(), WritingError>,
) -> Result<(), WritingError> {
    write.write_var_int(&VarInt(items.len() as i32))?;
    let mut bytes = Vec::new();
    for item in items {
        each(&mut bytes, item)?;
    }
    write.write_slice(&bytes)
}

fn write_text(write: &mut impl NetworkWriteExt, text: &TextComponent) -> Result<(), WritingError> {
    write.write_slice(&text.encode_for_version(&VERSION))
}

/// The 1.21.1 `EquipmentSlotGroup` id; 1.21.1 has no saddle group.
const fn slot_group_id(slot: &AttributeModifierSlot) -> Option<i32> {
    Some(match slot {
        AttributeModifierSlot::Any => 0,
        AttributeModifierSlot::MainHand => 1,
        AttributeModifierSlot::OffHand => 2,
        AttributeModifierSlot::Hand => 3,
        AttributeModifierSlot::Feet => 4,
        AttributeModifierSlot::Legs => 5,
        AttributeModifierSlot::Chest => 6,
        AttributeModifierSlot::Head => 7,
        AttributeModifierSlot::Armor => 8,
        AttributeModifierSlot::Body => 9,
        AttributeModifierSlot::Saddle => return None,
    })
}

fn strip_namespace(name: &str) -> &str {
    name.strip_prefix("minecraft:").unwrap_or(name)
}

/// Writes one component's value; returns false when it must be left out.
#[expect(clippy::too_many_lines)]
fn write_component(
    id: DataComponent,
    value: &dyn DataComponentImpl,
    w: &mut Vec<u8>,
) -> Result<bool, WritingError> {
    match id {
        // `Unbreakable`: show in tooltip
        DataComponent::Unbreakable => w.write_bool(true)?,
        // `ItemEnchantments`: the map, then show in tooltip
        DataComponent::Enchantments => {
            get::<EnchantmentsImpl>(value).serialize(w)?;
            w.write_bool(true)?;
        }
        DataComponent::StoredEnchantments => {
            get::<StoredEnchantmentsImpl>(value).serialize(w)?;
            w.write_bool(true)?;
        }
        // Pumpkin doesn't keep these in a sendable form: the block predicates of
        // `AdventureModePredicate`, which instrument, the recipes, the lock (a string in 1.21.1)
        // and the sherds.
        DataComponent::CanPlaceOn
        | DataComponent::CanBreak
        | DataComponent::Instrument
        | DataComponent::Recipes
        | DataComponent::Lock
        | DataComponent::PotDecorations => return Ok(false),
        // `ItemAttributeModifiers`: entries without a display, then show in tooltip
        DataComponent::AttributeModifiers => {
            let modifiers: Vec<_> = get::<AttributeModifiersImpl>(value)
                .attribute_modifiers
                .iter()
                .filter_map(|m| slot_group_id(&m.slot).map(|slot| (m, slot)))
                .collect();
            write_list(w, &modifiers, |w, (m, slot)| {
                w.write_var_int(&VarInt(m.r#type.id as i32))?;
                w.write_string(m.id)?;
                w.write_f64(m.amount)?;
                w.write_var_int(&VarInt(m.operation as i32))?;
                w.write_var_int(&VarInt(*slot))
            })?;
            w.write_bool(true)?;
        }
        // `CustomModelData`: one int
        DataComponent::CustomModelData => {
            let Some(value) = get::<CustomModelDataImpl>(value).floats.first() else {
                return Ok(false);
            };
            w.write_var_int(&VarInt(*value as i32))?;
        }
        // Default codecs: network NBT of the persistent form
        DataComponent::IntangibleProjectile
        | DataComponent::DebugStickState
        | DataComponent::MapDecorations
        | DataComponent::BucketEntityData => w.write_nbt(NbtTag::Compound(NbtCompound::new()))?,
        DataComponent::ContainerLoot => w.write_nbt(value.write_data())?,
        // `FoodProperties.DIRECT_STREAM_CODEC`
        DataComponent::Food => {
            let food = get::<FoodImpl>(value);
            w.write_var_int(&VarInt(food.nutrition))?;
            w.write_f32(food.saturation)?;
            w.write_bool(food.can_always_eat)?;
            // Eat time (vanilla's default), no converted item, no effects
            w.write_f32(1.6)?;
            w.write_bool(false)?;
            w.write_var_int(&VarInt(0))?;
        }
        // `Tool`: rules, default speed, damage per block
        DataComponent::Tool => {
            let tool = get::<ToolImpl>(value);
            write_list(w, &tool.rules, |w, rule| {
                serialize_idset(&rule.blocks, w)?;
                w.write_option(&rule.speed, |w, speed| w.write_f32(*speed))?;
                w.write_option(&rule.correct_for_drops, |w, correct| w.write_bool(*correct))
            })?;
            w.write_f32(tool.default_mining_speed)?;
            w.write_var_int(&VarInt(tool.damage_per_block as i32))?;
        }
        // `DyedItemColor`: color, then show in tooltip
        DataComponent::DyedColor => {
            w.write_i32(get::<DyedColorImpl>(value).rgb)?;
            w.write_bool(true)?;
        }
        DataComponent::ChargedProjectiles => {
            let stacks: Vec<_> = get::<ChargedProjectilesImpl>(value)
                .projectiles
                .iter()
                .filter_map(ItemStack::read_item_stack)
                .collect();
            write_list(w, &stacks, |w, stack| write_stack(stack, stack.item.id, w))?;
        }
        DataComponent::BundleContents => {
            write_list(w, &get::<BundleContentsImpl>(value).items, |w, stack| {
                write_stack(stack, stack.item.id, w)
            })?;
        }
        // `ItemContainerContents`: the slots in order, empty ones included
        DataComponent::Container => {
            let items = &get::<ContainerImpl>(value).items;
            let size = items
                .iter()
                .map(|(slot, _)| usize::from(*slot) + 1)
                .max()
                .unwrap_or(0);
            let mut slots = vec![ItemStack::EMPTY; size];
            for (slot, stack) in items {
                slots[usize::from(*slot)] = stack;
            }
            write_list(w, &slots, |w, stack| write_stack(stack, stack.item.id, w))?;
        }
        // `PotionContents`: potion, color, effects (the custom name came in 1.21.2)
        DataComponent::PotionContents => {
            let potion = get::<PotionContentsImpl>(value);
            w.write_option(&potion.potion_id, |w, id| w.write_var_int(&VarInt(*id)))?;
            w.write_option(&potion.custom_color, |w, color| w.write_i32(*color))?;
            serialize_status_effects(&potion.custom_effects, w)?;
        }
        // `ArmorTrim`: material and pattern holders, then show in tooltip
        DataComponent::Trim => {
            let trim = get::<TrimImpl>(value);
            let (NbtTag::String(material), NbtTag::String(pattern)) =
                (&trim.material, &trim.pattern)
            else {
                return Ok(false);
            };
            let (Some(material), Some(pattern)) = (
                TrimMaterial::from_name(strip_namespace(material)),
                TrimPattern::from_name(strip_namespace(pattern)),
            ) else {
                return Ok(false);
            };
            w.write_var_int(&VarInt(material.id() as i32 + 1))?;
            w.write_var_int(&VarInt(pattern.id() as i32 + 1))?;
            w.write_bool(true)?;
        }
        // `CustomData.STREAM_CODEC`: the whole tag, "id" included
        DataComponent::EntityData => {
            let Some(nbt) = &get::<EntityDataImpl>(value).nbt else {
                return Ok(false);
            };
            w.write_nbt(NbtTag::Compound(nbt.clone()))?;
        }
        DataComponent::BlockEntityData => {
            w.write_nbt(NbtTag::Compound(
                get::<BlockEntityDataImpl>(value).nbt.clone(),
            ))?;
        }
        // `JukeboxPlayable`: the song as a registry key, then show in tooltip
        DataComponent::JukeboxPlayable => {
            let song = get::<JukeboxPlayableImpl>(value).song;
            if song.is_empty() {
                return Ok(false);
            }
            w.write_bool(false)?;
            w.write_string(song)?;
            w.write_bool(true)?;
        }
        // `ResolvableProfile`: name, id, properties
        DataComponent::Profile => {
            let profile = get::<ProfileImpl>(value);
            w.write_option(&profile.name, |w, name| w.write_string_bounded(name, 16))?;
            w.write_option(&profile.id, |w, id| {
                let bits = id
                    .iter()
                    .fold(0u128, |acc, part| (acc << 32) | u128::from(*part as u32));
                w.write_uuid(&uuid::Uuid::from_u128(bits))
            })?;
            write_list(w, &profile.properties, |w, property| {
                w.write_string(&property.name)?;
                w.write_string(&property.value)?;
                w.write_option(&property.signature, |w, signature| {
                    w.write_string(signature)
                })
            })?;
        }
        // `BannerPatternLayers`: pattern holders and colors
        DataComponent::BannerPatterns => {
            let layers: Vec<_> = get::<BannerPatternsImpl>(value)
                .layers
                .iter()
                .filter_map(|layer| {
                    BannerPattern::from_name(strip_namespace(&layer.pattern))
                        .map(|pattern| (pattern.id() as i32 + 1, layer.color.id() as i32))
                })
                .collect();
            write_list(w, &layers, |w, (pattern, color)| {
                w.write_var_int(&VarInt(*pattern))?;
                w.write_var_int(&VarInt(*color))
            })?;
        }
        // Pumpkin doesn't keep the occupants
        DataComponent::Bees => w.write_var_int(&VarInt(0))?,
        DataComponent::Lore => {
            write_list(w, &get::<LoreImpl>(value).lines, |w, line| {
                write_text(w, line)
            })?;
        }
        // `WrittenBookContent`: filterable title, author, generation, filterable pages, resolved
        DataComponent::WrittenBookContent => {
            let book = get::<WrittenBookContentImpl>(value);
            w.write_string(&book.title)?;
            w.write_bool(false)?;
            w.write_string(&book.author)?;
            w.write_var_int(&VarInt(0))?;
            write_list(w, &book.pages, |w, page| {
                write_text(w, page)?;
                w.write_bool(false)
            })?;
            w.write_bool(true)?;
        }
        _ => serialize(id, value, w)?,
    }
    Ok(true)
}

fn read_count(read: &mut impl NetworkReadExt) -> Result<i32, ReadingError> {
    let count = read.get_var_int()?.0;
    if !(0..=MAX_LIST).contains(&count) {
        return Err(ReadingError::Message(format!("Bad list length {count}")));
    }
    Ok(count)
}

fn read_nbt(read: &mut impl NetworkReadExt) -> Result<NbtTag, ReadingError> {
    Ok(read
        .get_nbt_with_version(&VERSION)?
        .unwrap_or_else(|| NbtTag::Compound(NbtCompound::new())))
}

fn read_text(read: &mut impl NetworkReadExt) -> Result<TextComponent, ReadingError> {
    Ok(TextComponent::from_nbt(&read_nbt(read)?))
}

fn read_resource_location(read: &mut impl NetworkReadExt) -> Result<String, ReadingError> {
    Ok(read.get_str_bounded(32767)?.into())
}

/// Reads the id of a `holder` codec: the registry id plus one, or 0 when the value follows
/// inline (then `None`, and the caller reads the value).
fn read_holder(read: &mut impl NetworkReadExt) -> Result<Option<u32>, ReadingError> {
    let id = read.get_var_int()?.0;
    if id < 0 {
        return Err(ReadingError::Message("Negative holder id".into()));
    }
    Ok((id > 0).then(|| id as u32 - 1))
}

/// `SoundEvent.STREAM_CODEC`, read and dropped.
fn skip_sound_holder(read: &mut impl NetworkReadExt) -> Result<(), ReadingError> {
    if read_holder(read)?.is_none() {
        read_resource_location(read)?;
        if read.get_bool()? {
            read.get_f32()?;
        }
    }
    Ok(())
}

/// `BlockPredicate.STREAM_CODEC`, read and dropped.
fn skip_block_predicate(read: &mut impl NetworkReadExt) -> Result<(), ReadingError> {
    if read.get_bool()? {
        deserialize_idset::<pumpkin_data::Block>(read)?;
    }
    if read.get_bool()? {
        for _ in 0..read_count(read)? {
            read.get_str()?;
            if read.get_bool()? {
                read.get_str()?;
            } else {
                for _ in 0..2 {
                    if read.get_bool()? {
                        read.get_str()?;
                    }
                }
            }
        }
    }
    if read.get_bool()? {
        read_nbt(read)?;
    }
    Ok(())
}

/// `MobEffectInstance.STREAM_CODEC`, read and dropped.
fn skip_effect(read: &mut impl NetworkReadExt) -> Result<(), ReadingError> {
    read.get_var_int()?;
    loop {
        read.get_var_int()?;
        read.get_var_int()?;
        read.get_bool()?;
        read.get_bool()?;
        read.get_bool()?;
        if !read.get_bool()? {
            return Ok(());
        }
    }
}

fn read_string_option(read: &mut impl NetworkReadExt) -> Result<Option<String>, ReadingError> {
    if read.get_bool()? {
        Ok(Some(read.get_str()?.into()))
    } else {
        Ok(None)
    }
}

/// Reads one component's value; `None` when the server doesn't keep it.
#[expect(clippy::too_many_lines)]
fn read_component(
    id: DataComponent,
    read: &mut impl NetworkReadExt,
) -> Result<Option<Box<dyn DataComponentImpl>>, ReadingError> {
    Ok(Some(match id {
        DataComponent::Unbreakable => {
            read.get_bool()?;
            UnbreakableImpl.to_dyn()
        }
        DataComponent::Enchantments => {
            let enchantments = EnchantmentsImpl::deserialize(read)?;
            read.get_bool()?;
            enchantments.to_dyn()
        }
        DataComponent::StoredEnchantments => {
            let enchantments = StoredEnchantmentsImpl::deserialize(read)?;
            read.get_bool()?;
            enchantments.to_dyn()
        }
        DataComponent::CanPlaceOn | DataComponent::CanBreak => {
            for _ in 0..read_count(read)? {
                skip_block_predicate(read)?;
            }
            read.get_bool()?;
            return Ok(None);
        }
        DataComponent::AttributeModifiers => {
            for _ in 0..read_count(read)? {
                read.get_var_int()?;
                read_resource_location(read)?;
                read.get_f64()?;
                read.get_var_int()?;
                read.get_var_int()?;
            }
            read.get_bool()?;
            return Ok(None);
        }
        DataComponent::CustomModelData => CustomModelDataImpl {
            floats: vec![read.get_var_int()?.0 as f32],
            flags: Vec::new(),
            strings: Vec::new(),
            colors: Vec::new(),
        }
        .to_dyn(),
        DataComponent::HideAdditionalTooltip
        | DataComponent::HideTooltip
        | DataComponent::FireResistant => return Ok(None),
        DataComponent::IntangibleProjectile
        | DataComponent::DebugStickState
        | DataComponent::MapDecorations
        | DataComponent::BucketEntityData
        | DataComponent::Recipes
        | DataComponent::Lock
        | DataComponent::ContainerLoot
        | DataComponent::EntityData
        | DataComponent::BlockEntityData => return Ok(read_data(id, &read_nbt(read)?)),
        DataComponent::Food => {
            let nutrition = read.get_var_int()?.0;
            let saturation = read.get_f32()?;
            let can_always_eat = read.get_bool()?;
            read.get_f32()?;
            if read.get_bool()? {
                read_stack(read)?;
            }
            for _ in 0..read_count(read)? {
                skip_effect(read)?;
                read.get_f32()?;
            }
            FoodImpl {
                nutrition,
                saturation,
                can_always_eat,
            }
            .to_dyn()
        }
        DataComponent::Tool => {
            let mut rules = Vec::new();
            for _ in 0..read_count(read)? {
                let blocks = deserialize_idset(read)?;
                let speed = if read.get_bool()? {
                    Some(read.get_f32()?)
                } else {
                    None
                };
                let correct_for_drops = if read.get_bool()? {
                    Some(read.get_bool()?)
                } else {
                    None
                };
                rules.push(ToolRule {
                    blocks,
                    speed,
                    correct_for_drops,
                });
            }
            ToolImpl {
                rules: Cow::Owned(rules),
                default_mining_speed: read.get_f32()?,
                damage_per_block: read.get_var_int()?.0.max(0) as u32,
                can_destroy_blocks_in_creative: true,
            }
            .to_dyn()
        }
        DataComponent::DyedColor => {
            let rgb = read.get_i32()?;
            read.get_bool()?;
            DyedColorImpl { rgb }.to_dyn()
        }
        DataComponent::MapColor => {
            read.get_i32()?;
            return Ok(None);
        }
        DataComponent::ChargedProjectiles => {
            let mut projectiles = Vec::new();
            for _ in 0..read_count(read)? {
                let mut compound = NbtCompound::new();
                read_stack(read)?.write_item_stack(&mut compound);
                projectiles.push(compound);
            }
            ChargedProjectilesImpl { projectiles }.to_dyn()
        }
        DataComponent::BundleContents => {
            let mut items = Vec::new();
            for _ in 0..read_count(read)? {
                items.push(read_stack(read)?);
            }
            BundleContentsImpl { items }.to_dyn()
        }
        DataComponent::Container => {
            let mut items = Vec::new();
            for slot in 0..read_count(read)? {
                let stack = read_stack(read)?;
                if !stack.is_empty() {
                    items.push((slot as u8, stack));
                }
            }
            ContainerImpl { items }.to_dyn()
        }
        DataComponent::PotionContents => {
            let potion_id = if read.get_bool()? {
                Some(read.get_var_int()?.0)
            } else {
                None
            };
            let custom_color = if read.get_bool()? {
                Some(read.get_i32()?)
            } else {
                None
            };
            PotionContentsImpl {
                potion_id,
                custom_color,
                custom_effects: deserialize_status_effects(read)?,
                custom_name: None,
            }
            .to_dyn()
        }
        DataComponent::Trim => {
            let material = read_holder(read)?;
            if material.is_none() {
                // `TrimMaterial.DIRECT_STREAM_CODEC`
                read.get_str()?;
                read.get_var_int()?;
                read.get_f32()?;
                for _ in 0..read_count(read)? {
                    read.get_var_int()?;
                    read.get_str()?;
                }
                read_text(read)?;
            }
            let pattern = read_holder(read)?;
            if pattern.is_none() {
                // `TrimPattern.DIRECT_STREAM_CODEC`
                read_resource_location(read)?;
                read.get_var_int()?;
                read_text(read)?;
                read.get_bool()?;
            }
            read.get_bool()?;
            let material = material.and_then(|id| TrimMaterial::all().get(id as usize));
            let pattern = pattern.and_then(|id| TrimPattern::all().get(id as usize));
            let (Some(material), Some(pattern)) = (material, pattern) else {
                return Ok(None);
            };
            TrimImpl {
                material: NbtTag::String(format!("minecraft:{}", material.to_name()).into()),
                pattern: NbtTag::String(format!("minecraft:{}", pattern.to_name()).into()),
            }
            .to_dyn()
        }
        DataComponent::Instrument => {
            if read_holder(read)?.is_none() {
                // `Instrument.DIRECT_STREAM_CODEC`
                skip_sound_holder(read)?;
                read.get_var_int()?;
                read.get_f32()?;
            }
            InstrumentImpl.to_dyn()
        }
        DataComponent::JukeboxPlayable => {
            if read.get_bool()? {
                if read_holder(read)?.is_none() {
                    // `JukeboxSong.DIRECT_STREAM_CODEC`
                    skip_sound_holder(read)?;
                    read_text(read)?;
                    read.get_f32()?;
                    read.get_var_int()?;
                }
            } else {
                read_resource_location(read)?;
            }
            read.get_bool()?;
            return Ok(None);
        }
        DataComponent::Profile => {
            let name = if read.get_bool()? {
                Some(String::from(read.get_str_bounded(16)?))
            } else {
                None
            };
            let id = if read.get_bool()? {
                let bits = read.get_uuid()?.as_u128();
                Some([
                    (bits >> 96) as i32,
                    (bits >> 64) as i32,
                    (bits >> 32) as i32,
                    bits as i32,
                ])
            } else {
                None
            };
            let mut properties = Vec::new();
            for _ in 0..read_count(read)? {
                properties.push(ProfileProperty {
                    name: read.get_str()?.into(),
                    value: read.get_str()?.into(),
                    signature: read_string_option(read)?,
                });
            }
            ProfileImpl {
                name,
                id,
                properties,
                ..ProfileImpl::default()
            }
            .to_dyn()
        }
        DataComponent::BannerPatterns => {
            let mut layers = Vec::new();
            for _ in 0..read_count(read)? {
                let pattern = read_holder(read)?;
                if pattern.is_none() {
                    // `BannerPattern.DIRECT_STREAM_CODEC`
                    read_resource_location(read)?;
                    read.get_str()?;
                }
                let color = DyeColor::by_id(read.get_var_int()?.0 as u8);
                if let (Some(pattern), Some(color)) = (
                    pattern.and_then(|id| BannerPattern::all().get(id as usize)),
                    color,
                ) {
                    layers.push(BannerPatternLayer {
                        pattern: format!("minecraft:{}", pattern.to_name()),
                        color,
                    });
                }
            }
            BannerPatternsImpl { layers }.to_dyn()
        }
        DataComponent::Bees => {
            for _ in 0..read_count(read)? {
                read_nbt(read)?;
                read.get_var_int()?;
                read.get_var_int()?;
            }
            BeesImpl.to_dyn()
        }
        DataComponent::PotDecorations => {
            for _ in 0..read_count(read)? {
                read.get_var_int()?;
            }
            PotDecorationsImpl.to_dyn()
        }
        DataComponent::ItemName => {
            let name = match read_nbt(read)? {
                NbtTag::Compound(compound) => compound
                    .get_string("translate")
                    .or_else(|| compound.get_string("text"))
                    .unwrap_or_default()
                    .to_string(),
                NbtTag::String(text) => text.to_string(),
                _ => String::new(),
            };
            ItemNameImpl {
                name: Cow::Owned(name),
            }
            .to_dyn()
        }
        DataComponent::Lore => {
            let mut lines = Vec::new();
            for _ in 0..read_count(read)? {
                lines.push(read_text(read)?);
            }
            LoreImpl { lines }.to_dyn()
        }
        DataComponent::WritableBookContent => {
            let mut pages = Vec::new();
            for _ in 0..read_count(read)? {
                pages.push(read.get_str()?.into());
                read_string_option(read)?;
            }
            WritableBookContentImpl { pages }.to_dyn()
        }
        DataComponent::WrittenBookContent => {
            let title = read.get_str()?.into();
            read_string_option(read)?;
            let author = read.get_str()?.into();
            read.get_var_int()?;
            let mut pages = Vec::new();
            for _ in 0..read_count(read)? {
                pages.push(read_text(read)?);
                if read.get_bool()? {
                    read_text(read)?;
                }
            }
            read.get_bool()?;
            WrittenBookContentImpl {
                title,
                author,
                pages,
            }
            .to_dyn()
        }
        _ => deserialize(id, read)?,
    }))
}

#[cfg(test)]
mod tests {
    use std::borrow::Cow;

    use pumpkin_data::Enchantment;
    use pumpkin_data::data_component::DataComponent;
    use pumpkin_data::data_component_impl::{
        DataComponentImpl, EnchantmentsImpl, PotionContentsImpl, ToolImpl, get,
    };
    use pumpkin_data::item::Item;
    use pumpkin_data::item_stack::ItemStack;

    use super::{read_stack, write_stack};
    use crate::VarInt;
    use crate::ser::NetworkWriteExt;

    fn var_ints(values: &[i32]) -> Vec<u8> {
        let mut out = Vec::new();
        for value in values {
            out.write_var_int(&VarInt(*value)).unwrap();
        }
        out
    }

    #[test]
    fn enchantments_carry_the_tooltip_flag() {
        let stack = ItemStack::new_with_component(
            1,
            &Item::DIAMOND_SWORD,
            vec![(
                DataComponent::Enchantments,
                Some(
                    EnchantmentsImpl {
                        enchantment: Cow::Owned(vec![(&Enchantment::SHARPNESS, 3)]),
                    }
                    .to_dyn(),
                ),
            )],
        );
        let mut bytes = Vec::new();
        write_stack(&stack, stack.item.id, &mut bytes).unwrap();
        let mut expected = var_ints(&[
            1,
            i32::from(Item::DIAMOND_SWORD.id),
            1,
            0,
            i32::from(DataComponent::Enchantments.to_id()),
            1,
            i32::from(Enchantment::SHARPNESS.id),
            3,
        ]);
        expected.push(1);
        assert_eq!(bytes, expected);

        let read = read_stack(&mut bytes.as_slice()).unwrap();
        let enchantments = get::<EnchantmentsImpl>(read.patch[0].1.as_deref().unwrap());
        assert_eq!(enchantments.enchantment[0].1, 3);
    }

    #[test]
    fn potion_contents_have_no_custom_name() {
        let stack = ItemStack::new_with_component(
            1,
            &Item::POTION,
            vec![(
                DataComponent::PotionContents,
                Some(
                    PotionContentsImpl {
                        potion_id: Some(5),
                        custom_color: None,
                        custom_effects: Vec::new(),
                        custom_name: Some("x".into()),
                    }
                    .to_dyn(),
                ),
            )],
        );
        let mut bytes = Vec::new();
        write_stack(&stack, stack.item.id, &mut bytes).unwrap();
        let mut expected = var_ints(&[
            1,
            i32::from(Item::POTION.id),
            1,
            0,
            i32::from(DataComponent::PotionContents.to_id()),
        ]);
        // Some(potion 5), no color, no effects
        expected.extend_from_slice(&[1, 5, 0, 0]);
        assert_eq!(bytes, expected);
    }

    #[test]
    fn internal_components_are_not_sent() {
        let stack = ItemStack::new_with_component(
            1,
            &Item::DIAMOND_PICKAXE,
            vec![(
                DataComponent::Tool,
                Some(
                    ToolImpl {
                        rules: Cow::Owned(Vec::new()),
                        default_mining_speed: 1.0,
                        damage_per_block: 1,
                        can_destroy_blocks_in_creative: false,
                    }
                    .to_dyn(),
                ),
            )],
        );
        let mut bytes = Vec::new();
        write_stack(&stack, stack.item.id, &mut bytes).unwrap();
        // `Tool`: no rules, the default speed, damage per block, without the creative flag
        let mut expected = var_ints(&[
            1,
            i32::from(Item::DIAMOND_PICKAXE.id),
            1,
            0,
            i32::from(DataComponent::Tool.to_id()),
            0,
        ]);
        expected.extend_from_slice(&1.0f32.to_be_bytes());
        expected.push(1);
        assert_eq!(bytes, expected);

        let internal = ItemStack::new_with_component(
            1,
            &Item::DIAMOND_PICKAXE,
            vec![(DataComponent::Weapon, None)],
        );
        let mut bytes = Vec::new();
        write_stack(&internal, internal.item.id, &mut bytes).unwrap();
        assert_eq!(
            bytes,
            var_ints(&[1, i32::from(Item::DIAMOND_PICKAXE.id), 0, 0])
        );
    }
}
