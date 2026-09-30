use pumpkin_protocol::codec::recipe::DynamicRecipe;

pub trait RecipeProvider: Send + Sync {
    fn get_dynamic_recipes(&self) -> Vec<DynamicRecipe>;

    /// The result of a grid no recipe matches, from recipes defined in code (a mod's
    /// `CustomRecipe`). `grid` is row by row, `width` slots per row.
    fn match_special(
        &self,
        _width: usize,
        _grid: &[pumpkin_data::item_stack::ItemStack],
    ) -> Option<pumpkin_data::item_stack::ItemStack> {
        None
    }
}

#[derive(Clone, Copy)]
pub enum GenericRecipe<'a> {
    Vanilla(&'a pumpkin_data::recipes::CraftingRecipeTypes),
    Dynamic(&'a pumpkin_protocol::codec::recipe::OwnedCraftingRecipe),
}

#[derive(Clone, Copy)]
pub enum IngredientRef<'a> {
    Vanilla(&'a pumpkin_data::recipes::RecipeIngredientTypes),
    Dynamic(&'a pumpkin_protocol::codec::recipe::OwnedRecipeIngredient),
}

impl IngredientRef<'_> {
    #[must_use]
    pub fn match_item(&self, item: &pumpkin_data::item::Item) -> bool {
        match self {
            Self::Vanilla(v) => v.match_item(item),
            Self::Dynamic(d) => d.match_item(item),
        }
    }
}
