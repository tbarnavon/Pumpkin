use pumpkin_data::packet::clientbound::play::RECIPE;
use pumpkin_macros::java_packet;
use pumpkin_util::version::JavaMinecraftVersion;

use crate::{
    ClientPacket, VarInt,
    ser::{NetworkWriteExt, WritingError},
};

/// Unlocks or locks recipe book entries for clients before 1.21.2 (`ClientboundRecipePacket`).
#[java_packet(RECIPE)]
pub struct CRecipe<'a> {
    pub state: RecipeBookState,
    /// Open and filtering flags of the crafting, furnace, blast furnace and smoker books.
    pub book_settings: [bool; 8],
    pub recipes: &'a [&'a str],
    /// Recipes shown as new; only sent with [`RecipeBookState::Init`].
    pub highlights: &'a [&'a str],
}

#[derive(Clone, Copy)]
pub enum RecipeBookState {
    Init,
    Add,
    Remove,
}

impl ClientPacket for CRecipe<'_> {
    fn write_packet_data(
        &self,
        mut write: impl std::io::Write,
        _version: &JavaMinecraftVersion,
    ) -> Result<(), WritingError> {
        write.write_var_int(&VarInt(self.state as i32))?;
        for flag in self.book_settings {
            write.write_bool(flag)?;
        }
        write.write_var_int(&VarInt(self.recipes.len() as i32))?;
        for recipe in self.recipes {
            write.write_string(recipe)?;
        }
        if matches!(self.state, RecipeBookState::Init) {
            write.write_var_int(&VarInt(self.highlights.len() as i32))?;
            for recipe in self.highlights {
                write.write_string(recipe)?;
            }
        }
        Ok(())
    }
}
