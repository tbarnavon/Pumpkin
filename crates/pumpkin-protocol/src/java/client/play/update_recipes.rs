use std::sync::OnceLock;

use crate::codec::recipe::{
    DynamicRecipe, OwnedCookingRecipeType, OwnedCraftingRecipe, OwnedRecipeIngredient,
    OwnedRecipeResult,
};
use crate::{
    ClientPacket, ServerPacket, VarInt,
    ser::{NetworkReadSliceExt, NetworkWriteExt, ReadingError, WritingError},
};
use pumpkin_data::item::Item;
use pumpkin_data::packet::clientbound::play::UPDATE_RECIPES;
use pumpkin_data::recipe_sync::{
    RECIPE_SERIALIZERS, SYNCED_RECIPES, SyncedIngredient, SyncedRecipeData, SyncedResult,
};
use pumpkin_data::recipes::RecipeCategoryTypes;
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

/// The packet body for clients before 1.21.2: vanilla's recipes, then the dynamic ones.
///
/// Those clients build their recipe book and stonecutter lists from every recipe. Dynamic
/// recipes come from mods and plugins; one that can't be expressed (no id, an unknown item) is
/// left out.
pub fn encode_before_1_21_2(dynamic: &[DynamicRecipe]) -> Result<Vec<u8>, WritingError> {
    static VANILLA: OnceLock<Vec<u8>> = OnceLock::new();
    let vanilla = if let Some(vanilla) = VANILLA.get() {
        vanilla
    } else {
        let vanilla = encode_vanilla()?;
        VANILLA.get_or_init(|| vanilla)
    };
    let mut extra = Vec::new();
    let mut extra_count = 0;
    for recipe in dynamic {
        let mut bytes = Vec::new();
        if write_dynamic(&mut bytes, recipe).unwrap_or(false) {
            extra.extend_from_slice(&bytes);
            extra_count += 1;
        }
    }
    let mut write = Vec::new();
    write.write_var_int(&VarInt((SYNCED_RECIPES.len() + extra_count) as i32))?;
    write.extend_from_slice(vanilla);
    write.extend_from_slice(&extra);
    Ok(write)
}

/// The id of a dynamic recipe, if it has one.
#[must_use]
pub fn dynamic_recipe_id(recipe: &DynamicRecipe) -> Option<&str> {
    match recipe {
        DynamicRecipe::Crafting(
            OwnedCraftingRecipe::Shaped { recipe_id, .. }
            | OwnedCraftingRecipe::Shapeless { recipe_id, .. },
        ) => recipe_id.as_deref(),
        DynamicRecipe::Cooking(
            OwnedCookingRecipeType::Blasting(cooking)
            | OwnedCookingRecipeType::Smelting(cooking)
            | OwnedCookingRecipeType::Smoking(cooking)
            | OwnedCookingRecipeType::CampfireCooking(cooking),
        ) => Some(&cooking.recipe_id),
        DynamicRecipe::Brewing(_) => None,
    }
}

/// Writes a dynamic recipe in its 1.21.1 serializer's layout; false when it can't be sent.
fn write_dynamic(write: &mut Vec<u8>, recipe: &DynamicRecipe) -> Result<bool, WritingError> {
    let Some(id) = dynamic_recipe_id(recipe) else {
        return Ok(false);
    };
    match recipe {
        DynamicRecipe::Crafting(OwnedCraftingRecipe::Shaped {
            category,
            group,
            show_notification,
            key,
            pattern,
            result,
            ..
        }) => {
            write_header(write, id, "minecraft:crafting_shaped")?;
            write.write_string(group.as_deref().unwrap_or(""))?;
            write.write_var_int(&VarInt(crafting_category(category)))?;
            let rows = shrink(pattern);
            let width = rows.first().map_or(0, |row| row.chars().count());
            write.write_var_int(&VarInt(width as i32))?;
            write.write_var_int(&VarInt(rows.len() as i32))?;
            for symbol in rows.iter().flat_map(|row| row.chars()) {
                match key.iter().find(|(k, _)| *k == symbol) {
                    Some((_, ingredient)) => write_owned_ingredient(write, ingredient)?,
                    None => write.write_var_int(&VarInt(0))?,
                }
            }
            write_owned_result(write, result)?;
            write.write_bool(*show_notification)?;
        }
        DynamicRecipe::Crafting(OwnedCraftingRecipe::Shapeless {
            category,
            group,
            ingredients,
            result,
            ..
        }) => {
            write_header(write, id, "minecraft:crafting_shapeless")?;
            write.write_string(group.as_deref().unwrap_or(""))?;
            write.write_var_int(&VarInt(crafting_category(category)))?;
            write.write_var_int(&VarInt(ingredients.len() as i32))?;
            for ingredient in ingredients {
                write_owned_ingredient(write, ingredient)?;
            }
            write_owned_result(write, result)?;
        }
        DynamicRecipe::Cooking(cooking) => {
            let (serializer, cooking) = match cooking {
                OwnedCookingRecipeType::Blasting(c) => ("minecraft:blasting", c),
                OwnedCookingRecipeType::Smelting(c) => ("minecraft:smelting", c),
                OwnedCookingRecipeType::Smoking(c) => ("minecraft:smoking", c),
                OwnedCookingRecipeType::CampfireCooking(c) => ("minecraft:campfire_cooking", c),
            };
            write_header(write, id, serializer)?;
            write.write_string(cooking.group.as_deref().unwrap_or(""))?;
            // CookingBookCategory: food, blocks, misc.
            write.write_var_int(&VarInt(match cooking.category {
                RecipeCategoryTypes::Food => 0,
                RecipeCategoryTypes::Blocks => 1,
                _ => 2,
            }))?;
            write_owned_ingredient(write, &cooking.ingredient)?;
            write_owned_result(write, &cooking.result)?;
            write.write_f32_be(cooking.experience)?;
            write.write_var_int(&VarInt(cooking.cooking_time))?;
        }
        DynamicRecipe::Brewing(_) => return Ok(false),
    }
    Ok(true)
}

fn write_header(write: &mut Vec<u8>, id: &str, serializer: &str) -> Result<(), WritingError> {
    let serializer = RECIPE_SERIALIZERS
        .iter()
        .position(|name| *name == serializer)
        .ok_or_else(|| WritingError::Message(format!("Unknown recipe serializer {serializer}")))?;
    write.write_string(id)?;
    write.write_var_int(&VarInt(serializer as i32))
}

/// `CraftingBookCategory`: building, redstone, equipment, misc.
const fn crafting_category(category: &RecipeCategoryTypes) -> i32 {
    match category {
        RecipeCategoryTypes::Building => 0,
        RecipeCategoryTypes::Restone => 1,
        RecipeCategoryTypes::Equipment => 2,
        _ => 3,
    }
}

fn write_owned_ingredient(
    write: &mut Vec<u8>,
    ingredient: &OwnedRecipeIngredient,
) -> Result<(), WritingError> {
    let items = ingredient.matching_items();
    write.write_var_int(&VarInt(items.len() as i32))?;
    for item in items {
        write_stack(write, i32::from(item.id), 1)?;
    }
    Ok(())
}

fn write_owned_result(write: &mut Vec<u8>, result: &OwnedRecipeResult) -> Result<(), WritingError> {
    write_stack(write, item_id(&result.item_id)?, i32::from(result.count))
}

/// Vanilla's `ShapedRecipePattern.shrink`: drops blank rows and columns around the pattern.
fn shrink(rows: &[String]) -> Vec<&str> {
    let first = rows
        .iter()
        .map(|row| row.find(|c| c != ' ').unwrap_or(row.len()))
        .min()
        .unwrap_or(0);
    let Some(last) = rows.iter().filter_map(|row| row.rfind(|c| c != ' ')).max() else {
        return Vec::new();
    };
    let start = rows.iter().take_while(|row| row.trim().is_empty()).count();
    let end = rows.len()
        - rows
            .iter()
            .rev()
            .take_while(|row| row.trim().is_empty())
            .count();
    rows[start..end]
        .iter()
        .map(|row| row.get(first..=last).unwrap_or(row))
        .collect()
}

/// Vanilla's recipe entries (without the count), in `SYNCED_RECIPES` order.
fn encode_vanilla() -> Result<Vec<u8>, WritingError> {
    let mut write = Vec::new();
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
    Item::from_registry_key(name.strip_prefix("minecraft:").unwrap_or(name))
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ser::NetworkReadExt;

    fn shaped(recipe_id: Option<&str>) -> DynamicRecipe {
        DynamicRecipe::Crafting(OwnedCraftingRecipe::Shaped {
            recipe_id: recipe_id.map(str::to_string),
            category: RecipeCategoryTypes::Building,
            group: None,
            show_notification: true,
            key: vec![('#', OwnedRecipeIngredient::Simple("minecraft:stick".into()))],
            pattern: vec!["   ".into(), " # ".into(), " # ".into()],
            result: OwnedRecipeResult {
                item_id: "minecraft:torch".into(),
                count: 4,
            },
        })
    }

    #[test]
    fn dynamic_recipes_follow_vanilla() {
        let vanilla = encode_before_1_21_2(&[]).unwrap();
        let both = encode_before_1_21_2(&[shaped(Some("testmod:torch")), shaped(None)]).unwrap();
        let count = |data: &[u8]| (&mut &data[..]).get_var_int().unwrap().0 as usize;
        assert_eq!(count(&vanilla), SYNCED_RECIPES.len());
        // The recipe without an id is left out.
        assert_eq!(count(&both), SYNCED_RECIPES.len() + 1);

        let mut expected = Vec::new();
        expected.write_string("testmod:torch").unwrap();
        let shaped_id = RECIPE_SERIALIZERS
            .iter()
            .position(|name| *name == "minecraft:crafting_shaped")
            .unwrap();
        expected.write_var_int(&VarInt(shaped_id as i32)).unwrap();
        // no group, building, the pattern shrunk to 1x2
        expected.write_string("").unwrap();
        for value in [0, 1, 2] {
            expected.write_var_int(&VarInt(value)).unwrap();
        }
        for _ in 0..2 {
            expected.write_var_int(&VarInt(1)).unwrap();
            write_stack(&mut expected, item_id("stick").unwrap(), 1).unwrap();
        }
        write_stack(&mut expected, item_id("torch").unwrap(), 4).unwrap();
        expected.write_bool(true).unwrap();
        assert!(both.ends_with(&expected));
    }
}
