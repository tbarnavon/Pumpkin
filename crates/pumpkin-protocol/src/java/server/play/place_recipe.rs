use crate::{
    ServerPacket,
    ser::{NetworkReadExt, NetworkReadSliceExt, ReadingError},
};
use pumpkin_data::packet::serverbound::play::PLACE_RECIPE;
use pumpkin_data::recipe_sync::{SYNCED_RECIPES, SyncedRecipeData};
use pumpkin_macros::java_packet;
use pumpkin_util::version::JavaMinecraftVersion;

use crate::VarInt;

#[java_packet(PLACE_RECIPE)]
pub struct SPlaceRecipe {
    pub container_id: i8,
    pub recipe_display_id: VarInt,
    pub use_max_items: bool,
}

/// The display id later versions give the crafting recipe `recipe`, or -1 if there is none.
///
/// Crafting recipes are numbered by id, as in `RECIPES_CRAFTING`.
fn crafting_display_id(recipe: &str) -> i32 {
    SYNCED_RECIPES
        .iter()
        .filter(|r| {
            matches!(
                r.data,
                SyncedRecipeData::Shaped { .. } | SyncedRecipeData::Shapeless { .. }
            )
        })
        .position(|r| r.id == recipe)
        .map_or(-1, |index| index as i32)
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
                recipe_display_id: VarInt(crafting_display_id(recipe)),
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
