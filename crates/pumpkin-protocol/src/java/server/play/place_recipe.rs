use crate::{
    ServerPacket,
    ser::{NetworkReadExt, NetworkReadSliceExt, ReadingError},
};
use pumpkin_data::packet::serverbound::play::PLACE_RECIPE;
use pumpkin_data::recipe_sync::{SYNCED_RECIPES, SyncedRecipeData};
use pumpkin_data::recipes::{CookingRecipeType, RECIPES_COOKING};
use pumpkin_macros::java_packet;
use pumpkin_util::version::JavaMinecraftVersion;

use crate::VarInt;

#[java_packet(PLACE_RECIPE)]
pub struct SPlaceRecipe {
    pub container_id: i8,
    pub recipe_display_id: VarInt,
    pub use_max_items: bool,
}

fn crafting_recipes() -> impl Iterator<Item = &'static str> {
    SYNCED_RECIPES
        .iter()
        .filter(|r| {
            matches!(
                r.data,
                SyncedRecipeData::Shaped { .. } | SyncedRecipeData::Shapeless { .. }
            )
        })
        .map(|r| r.id)
}

const fn cooking_recipe_id(recipe: &CookingRecipeType) -> &'static str {
    match recipe {
        CookingRecipeType::Blasting(r)
        | CookingRecipeType::Smelting(r)
        | CookingRecipeType::Smoking(r)
        | CookingRecipeType::CampfireCooking(r) => r.recipe_id,
    }
}

/// The display id later versions give the recipe `recipe`, or -1 if there is none.
///
/// Crafting recipes come first, numbered by id as in `RECIPES_CRAFTING`, then the cooking
/// recipes in `RECIPES_COOKING`'s order.
fn display_id(recipe: &str) -> i32 {
    if let Some(index) = crafting_recipes().position(|id| id == recipe) {
        return index as i32;
    }
    RECIPES_COOKING
        .iter()
        .position(|r| cooking_recipe_id(r) == recipe)
        .map_or(-1, |index| (crafting_recipes().count() + index) as i32)
}

/// The recipe id behind a display id from [`SPlaceRecipe`], for clients before 1.21.2, which
/// name recipes by id (the ghost recipe packet).
#[must_use]
pub fn recipe_id_of_display(display_id: usize) -> Option<&'static str> {
    let crafting_count = crafting_recipes().count();
    if display_id < crafting_count {
        crafting_recipes().nth(display_id)
    } else {
        RECIPES_COOKING
            .get(display_id - crafting_count)
            .map(cooking_recipe_id)
    }
}

impl<'a> ServerPacket<'a> for SPlaceRecipe {
    fn read(bytebuf: &mut &'a [u8], version: &JavaMinecraftVersion) -> Result<Self, ReadingError> {
        if *version >= JavaMinecraftVersion::V_1_21_2 {
            let container_id = bytebuf.get_container_id(version)?.0 as i8;
            let recipe_display_id = bytebuf.get_var_int()?;
            let use_max_items = bytebuf.get_bool()?;
            Ok(Self {
                container_id,
                recipe_display_id,
                use_max_items,
            })
        } else if *version >= JavaMinecraftVersion::V_1_13 {
            let container_id = bytebuf.get_i8()?;
            let recipe = bytebuf.get_str_borrowed()?;
            let use_max_items = bytebuf.get_bool()?;
            Ok(Self {
                container_id,
                recipe_display_id: VarInt(display_id(recipe)),
                use_max_items,
            })
        } else {
            let container_id = bytebuf.get_i8()?;
            let recipe_display_id = bytebuf.get_var_int()?;
            let use_max_items = bytebuf.get_bool()?;
            Ok(Self {
                container_id,
                recipe_display_id,
                use_max_items,
            })
        }
    }
}

impl crate::ClientPacket for SPlaceRecipe {
    fn write_packet_data(
        &self,
        mut write: impl std::io::Write,
        _version: &JavaMinecraftVersion,
    ) -> Result<(), crate::ser::WritingError> {
        use crate::ser::NetworkWriteExt;
        write.write_i8(self.container_id)?;
        write.write_var_int(&self.recipe_display_id)?;
        write.write_bool(self.use_max_items)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use pumpkin_data::recipes::{CraftingRecipeTypes, RECIPES_COOKING, RECIPES_CRAFTING};

    use super::{display_id, recipe_id_of_display};

    #[test]
    fn display_ids_line_up_with_the_recipe_lists() {
        // The handler numbers crafting recipes without the special ones, then cooking ones.
        let crafting = RECIPES_CRAFTING
            .iter()
            .filter(|r| {
                !matches!(
                    r,
                    CraftingRecipeTypes::CraftingSpecial
                        | CraftingRecipeTypes::CraftingDecoratedPot { .. }
                )
            })
            .count();
        assert_eq!(display_id("minecraft:baked_potato"), crafting as i32);
        assert!(RECIPES_COOKING.len() > 1);
        assert_eq!(
            recipe_id_of_display(crafting),
            Some("minecraft:baked_potato")
        );
        let planks = display_id("minecraft:oak_planks");
        assert!(planks >= 0 && (planks as usize) < crafting);
        assert_eq!(
            recipe_id_of_display(planks as usize),
            Some("minecraft:oak_planks")
        );
    }
}
