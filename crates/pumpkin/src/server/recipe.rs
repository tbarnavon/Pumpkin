use std::sync::RwLock;

use pumpkin_inventory::crafting::recipe_provider::RecipeProvider;
pub use pumpkin_protocol::codec::recipe::DynamicRecipe;

/// Crafts grids no recipe matches (a plugin's `handle-crafting`).
pub trait SpecialCraftingHandler: Send + Sync {
    fn craft(
        &self,
        width: usize,
        grid: &[pumpkin_data::item_stack::ItemStack],
    ) -> Option<pumpkin_data::item_stack::ItemStack>;

    /// Whether `other` is the same handler, registered again (after a plugin restart).
    fn same_as(&self, other: &dyn std::any::Any) -> bool;

    fn as_any(&self) -> &dyn std::any::Any;
}

pub struct RecipeManager {
    dynamic_recipes: RwLock<Vec<DynamicRecipe>>,
    special: RwLock<Vec<std::sync::Arc<dyn SpecialCraftingHandler>>>,
}

impl Default for RecipeManager {
    fn default() -> Self {
        Self::new()
    }
}

impl RecipeManager {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            dynamic_recipes: RwLock::new(Vec::new()),
            special: RwLock::new(Vec::new()),
        }
    }

    /// Adds a special crafting handler, replacing the same one registered earlier.
    pub fn add_special(&self, handler: std::sync::Arc<dyn SpecialCraftingHandler>) {
        let mut special = self
            .special
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        special.retain(|existing| !existing.same_as(handler.as_any()));
        special.push(handler);
    }

    pub fn add_recipe(&self, recipe: DynamicRecipe) {
        let mut recipes = self
            .dynamic_recipes
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        recipes.push(recipe);
    }

    pub fn add_recipes(&self, new_recipes: impl IntoIterator<Item = DynamicRecipe>) {
        let mut recipes = self
            .dynamic_recipes
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        recipes.extend(new_recipes);
    }

    pub fn set_recipes(&self, new_recipes: Vec<DynamicRecipe>) {
        let mut recipes = self
            .dynamic_recipes
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        *recipes = new_recipes;
    }

    pub fn clear(&self) {
        let mut recipes = self
            .dynamic_recipes
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        recipes.clear();
    }

    pub fn get_dynamic_recipes_internal(&self) -> Vec<DynamicRecipe> {
        self.dynamic_recipes
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clone()
    }
}

impl RecipeProvider for RecipeManager {
    fn get_dynamic_recipes(&self) -> Vec<DynamicRecipe> {
        self.dynamic_recipes
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clone()
    }

    fn match_special(
        &self,
        width: usize,
        grid: &[pumpkin_data::item_stack::ItemStack],
    ) -> Option<pumpkin_data::item_stack::ItemStack> {
        let handlers = self
            .special
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clone();
        handlers
            .iter()
            .find_map(|handler| handler.craft(width, grid))
    }
}
