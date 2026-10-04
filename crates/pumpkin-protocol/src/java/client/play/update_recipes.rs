use std::sync::OnceLock;

use crate::{
    ClientPacket, ServerPacket, VarInt,
    ser::{NetworkReadSliceExt, NetworkWriteExt, ReadingError, WritingError},
};
use pumpkin_data::item::Item;
use pumpkin_data::packet::clientbound::play::UPDATE_RECIPES;
use pumpkin_data::recipe_sync::{SYNCED_RECIPES, SyncedIngredient, SyncedRecipeData, SyncedResult};
use pumpkin_data::tag::{RegistryKey, get_tag_ids};
use pumpkin_macros::java_packet;
use pumpkin_util::version::JavaMinecraftVersion;

#[java_packet(UPDATE_RECIPES)]
pub struct CUpdateRecipes<'a> {
    pub raw_data: &'a [u8],
}

impl<'a> CUpdateRecipes<'a> {
    #[must_use]
    pub const fn new(raw_data: &'a [u8]) -> Self {
        Self { raw_data }
    }
}

impl CUpdateRecipes<'static> {
    /// Every recipe in the layout clients before 1.21.2 read: they build their recipe book and
    /// stonecutter lists from it.
    pub fn before_1_21_2() -> Result<Self, WritingError> {
        static DATA: OnceLock<Vec<u8>> = OnceLock::new();
        if let Some(data) = DATA.get() {
            return Ok(Self::new(data));
        }
        let data = encode_recipes()?;
        Ok(Self::new(DATA.get_or_init(|| data)))
    }
}

fn encode_recipes() -> Result<Vec<u8>, WritingError> {
    let mut write = Vec::new();
    write.write_var_int(&VarInt(SYNCED_RECIPES.len() as i32))?;
    for recipe in SYNCED_RECIPES {
        write.write_string(recipe.id)?;
        write.write_var_int(&VarInt(recipe.serializer))?;
        match recipe.data {
            SyncedRecipeData::Shaped {
                group,
                category,
                width,
                height,
                ingredients,
                result,
                show_notification,
            } => {
                write.write_string(group)?;
                write.write_var_int(&VarInt(category))?;
                write.write_var_int(&VarInt(width))?;
                write.write_var_int(&VarInt(height))?;
                for ingredient in ingredients {
                    write_ingredient(&mut write, ingredient)?;
                }
                write_result(&mut write, result)?;
                write.write_bool(show_notification)?;
            }
            SyncedRecipeData::Shapeless {
                group,
                category,
                ingredients,
                result,
            } => {
                write.write_string(group)?;
                write.write_var_int(&VarInt(category))?;
                write.write_var_int(&VarInt(ingredients.len() as i32))?;
                for ingredient in ingredients {
                    write_ingredient(&mut write, ingredient)?;
                }
                write_result(&mut write, result)?;
            }
            SyncedRecipeData::Cooking {
                group,
                category,
                ingredient,
                result,
                experience,
                cooking_time,
            } => {
                write.write_string(group)?;
                write.write_var_int(&VarInt(category))?;
                write_ingredient(&mut write, ingredient)?;
                write_result(&mut write, result)?;
                write.write_f32_be(experience)?;
                write.write_var_int(&VarInt(cooking_time))?;
            }
            SyncedRecipeData::Stonecutting {
                group,
                ingredient,
                result,
            } => {
                write.write_string(group)?;
                write_ingredient(&mut write, ingredient)?;
                write_result(&mut write, result)?;
            }
            SyncedRecipeData::SmithingTransform {
                template,
                base,
                addition,
                result,
            } => {
                write_ingredient(&mut write, template)?;
                write_ingredient(&mut write, base)?;
                write_ingredient(&mut write, addition)?;
                write_result(&mut write, result)?;
            }
            SyncedRecipeData::SmithingTrim {
                template,
                base,
                addition,
            } => {
                write_ingredient(&mut write, template)?;
                write_ingredient(&mut write, base)?;
                write_ingredient(&mut write, addition)?;
            }
            SyncedRecipeData::Special { category } => {
                write.write_var_int(&VarInt(category))?;
            }
        }
    }
    Ok(write)
}

fn item_id(name: &str) -> Result<i32, WritingError> {
    Item::from_registry_key(name)
        .map(|item| i32::from(item.id))
        .ok_or_else(|| WritingError::Message(format!("Unknown recipe item {name}")))
}

/// Writes an item stack without components (`ItemStack.STREAM_CODEC`).
fn write_stack(write: &mut Vec<u8>, item_id: i32, count: i32) -> Result<(), WritingError> {
    write.write_var_int(&VarInt(count))?;
    write.write_var_int(&VarInt(item_id))?;
    // No components added or removed.
    write.write_var_int(&VarInt(0))?;
    write.write_var_int(&VarInt(0))
}

fn write_result(write: &mut Vec<u8>, result: SyncedResult) -> Result<(), WritingError> {
    write_stack(write, item_id(result.id)?, result.count)
}

/// Writes an ingredient as the list of items it accepts, tags expanded.
fn write_ingredient(write: &mut Vec<u8>, parts: &[SyncedIngredient]) -> Result<(), WritingError> {
    let mut items = Vec::new();
    for part in parts {
        match *part {
            SyncedIngredient::Item(name) => items.push(item_id(name)?),
            SyncedIngredient::Tag(tag) => items.extend(
                get_tag_ids(RegistryKey::Item, tag)
                    .unwrap_or_default()
                    .iter()
                    .map(|id| i32::from(*id)),
            ),
        }
    }
    write.write_var_int(&VarInt(items.len() as i32))?;
    for item in items {
        write_stack(write, item, 1)?;
    }
    Ok(())
}

impl ClientPacket for CUpdateRecipes<'_> {
    fn write_packet_data(
        &self,
        mut write: impl std::io::Write,
        _version: &JavaMinecraftVersion,
    ) -> Result<(), WritingError> {
        write.write_slice(self.raw_data)?;
        Ok(())
    }
}

impl<'a> ServerPacket<'a> for CUpdateRecipes<'a> {
    fn read(bytebuf: &mut &'a [u8], _version: &JavaMinecraftVersion) -> Result<Self, ReadingError> {
        let raw_data = bytebuf.read_remaining_slice_borrowed(usize::MAX)?;
        Ok(Self { raw_data })
    }
}
