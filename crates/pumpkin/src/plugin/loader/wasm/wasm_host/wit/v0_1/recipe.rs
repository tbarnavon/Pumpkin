use crate::plugin::loader::wasm::wasm_host::state::PluginHostState;
use crate::plugin::loader::wasm::wasm_host::wit::v0_1::pumpkin::plugin::recipe::{
    CookingRecipe as WitCookingRecipe, CookingType as WitCookingType, Host as RecipeHost,
    HostRecipeManager, Ingredient as WitIngredient, RecipeCategory as WitRecipeCategory,
    RecipeManager as WitRecipeManager, ShapedRecipe as WitShapedRecipe,
    ShapelessRecipe as WitShapelessRecipe,
};
use pumpkin_data::recipes::RecipeCategoryTypes;
use pumpkin_protocol::codec::recipe::{
    DynamicRecipe, OwnedCookingRecipe, OwnedCookingRecipeType, OwnedCraftingRecipe,
    OwnedRecipeIngredient, OwnedRecipeResult,
};
use wasmtime::component::Resource;

impl RecipeHost for PluginHostState {}

/// A crafting grid given by a plugin, for `match_crafting_recipe`.
struct Grid {
    width: usize,
    items: Vec<pumpkin_data::item_stack::ItemStack>,
}

impl pumpkin_inventory::Clearable for Grid {
    fn clear(&self) {}
}

impl pumpkin_inventory::inventory::Inventory for Grid {
    fn size(&self) -> usize {
        self.items.len()
    }

    fn is_empty(&self) -> bool {
        self.items
            .iter()
            .all(pumpkin_data::item_stack::ItemStack::is_empty)
    }

    fn get_stack(&self, slot: usize) -> pumpkin_data::item_stack::ItemStack {
        self.items
            .get(slot)
            .cloned()
            .unwrap_or_else(|| pumpkin_data::item_stack::ItemStack::EMPTY.clone())
    }

    fn remove_stack(&self, slot: usize) -> pumpkin_data::item_stack::ItemStack {
        self.get_stack(slot)
    }

    fn remove_stack_specific(
        &self,
        slot: usize,
        _amount: u8,
    ) -> pumpkin_data::item_stack::ItemStack {
        self.get_stack(slot)
    }

    fn set_stack(&self, _slot: usize, _stack: pumpkin_data::item_stack::ItemStack) {}

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl pumpkin_inventory::crafting::recipes::RecipeInputInventory for Grid {
    fn get_width(&self) -> usize {
        self.width
    }

    fn get_height(&self) -> usize {
        self.items.len().div_ceil(self.width.max(1))
    }
}

/// A recipe result as a new item stack resource.
fn result_stack(
    state: &mut PluginHostState,
    item_id: &str,
    count: u8,
) -> wasmtime::Result<Option<Resource<super::pumpkin::plugin::item_stack::ItemStack>>> {
    let Some(item) = pumpkin_data::item::Item::from_registry_key(
        item_id.strip_prefix("minecraft:").unwrap_or(item_id),
    )
    .or_else(|| pumpkin_data::item::Item::from_registry_key(item_id)) else {
        return Ok(None);
    };
    state
        .add::<super::pumpkin::plugin::item_stack::ItemStack>(std::sync::Arc::new(
            tokio::sync::Mutex::new(pumpkin_data::item_stack::ItemStack::new(count, item)),
        ))
        .map(Some)
}

impl HostRecipeManager for PluginHostState {
    async fn match_crafting(
        &mut self,
        _res: Resource<WitRecipeManager>,
        width: u32,
        grid: Vec<Option<Resource<super::pumpkin::plugin::item_stack::ItemStack>>>,
    ) -> wasmtime::Result<Option<Resource<super::pumpkin::plugin::item_stack::ItemStack>>> {
        let width = width as usize;
        if width == 0 || width > 3 || grid.len() > 9 {
            return Ok(None);
        }
        let mut handles = Vec::with_capacity(grid.len());
        for slot in &grid {
            handles.push(
                slot.as_ref()
                    .map(|slot| self.get(slot).cloned())
                    .transpose()?,
            );
        }
        let mut items = Vec::with_capacity(handles.len());
        for handle in handles {
            items.push(match handle {
                Some(handle) => handle.lock().await.clone(),
                None => pumpkin_data::item_stack::ItemStack::EMPTY.clone(),
            });
        }
        let server = self
            .server
            .clone()
            .ok_or_else(|| wasmtime::Error::msg("Server not available"))?;
        let provider: &dyn pumpkin_inventory::crafting::recipe_provider::RecipeProvider =
            server.recipe_manager.as_ref();
        let Some(result) =
            pumpkin_inventory::crafting::crafting_screen_handler::match_crafting_recipe(
                &Grid { width, items },
                Some(provider),
            )
        else {
            return Ok(None);
        };
        result_stack(self, &result.item_id, result.count)
    }

    async fn match_cooking(
        &mut self,
        _res: Resource<WitRecipeManager>,
        station_type: WitCookingType,
        input: Resource<super::pumpkin::plugin::item_stack::ItemStack>,
    ) -> wasmtime::Result<Option<Resource<super::pumpkin::plugin::item_stack::ItemStack>>> {
        use pumpkin_data::recipes::CookingRecipeKind;
        let input = self.get(&input)?.clone();
        let item = input.lock().await.item;
        let kind = match station_type {
            WitCookingType::Smelting => CookingRecipeKind::Smelting,
            WitCookingType::Blasting => CookingRecipeKind::Blasting,
            WitCookingType::Smoking => CookingRecipeKind::Smoking,
            WitCookingType::Campfire => CookingRecipeKind::CampfireCooking,
        };
        let Some(recipe) = pumpkin_data::recipes::get_cooking_recipe_with_ingredient(item, kind)
        else {
            return Ok(None);
        };
        result_stack(self, recipe.result.id, recipe.result.count)
    }

    async fn register_shaped(
        &mut self,
        _res: Resource<WitRecipeManager>,
        id: String,
        recipe: WitShapedRecipe,
    ) -> wasmtime::Result<()> {
        let result_stack = self.take(recipe.output)?;
        let result_stack = result_stack.lock().await;

        let category = recipe
            .category
            .map_or(RecipeCategoryTypes::Misc, to_data_category);

        let owned_recipe = OwnedCraftingRecipe::Shaped {
            recipe_id: Some(id),
            category,
            group: recipe.group,
            show_notification: recipe.show_notification.unwrap_or(true),
            key: recipe
                .key
                .into_iter()
                .map(|(k, ing)| (k.chars().next().unwrap_or(' '), to_owned_ingredient(ing)))
                .collect(),
            pattern: recipe.pattern,
            result: OwnedRecipeResult {
                item_id: result_stack.item.registry_key.to_string(),
                count: result_stack.item_count,
            },
        };

        let server = self
            .server
            .as_ref()
            .ok_or_else(|| wasmtime::Error::msg("Server not available"))?;
        server
            .recipe_manager
            .add_recipe(DynamicRecipe::Crafting(owned_recipe));
        Ok(())
    }

    async fn register_shapeless(
        &mut self,
        _res: Resource<WitRecipeManager>,
        id: String,
        recipe: WitShapelessRecipe,
    ) -> wasmtime::Result<()> {
        let result_stack = self.take(recipe.output)?;
        let result_stack = result_stack.lock().await;

        let category = recipe
            .category
            .map_or(RecipeCategoryTypes::Misc, to_data_category);

        let owned_recipe = OwnedCraftingRecipe::Shapeless {
            recipe_id: Some(id),
            category,
            group: recipe.group,
            ingredients: recipe
                .ingredients
                .into_iter()
                .map(to_owned_ingredient)
                .collect(),
            result: OwnedRecipeResult {
                item_id: result_stack.item.registry_key.to_string(),
                count: result_stack.item_count,
            },
        };

        let server = self
            .server
            .as_ref()
            .ok_or_else(|| wasmtime::Error::msg("Server not available"))?;
        server
            .recipe_manager
            .add_recipe(DynamicRecipe::Crafting(owned_recipe));
        Ok(())
    }

    async fn register_cooking(
        &mut self,
        _res: Resource<WitRecipeManager>,
        id: String,
        station_type: WitCookingType,
        recipe: WitCookingRecipe,
    ) -> wasmtime::Result<()> {
        let result_stack = self.take(recipe.output)?;
        let result_stack = result_stack.lock().await;

        let category = recipe
            .category
            .map_or(RecipeCategoryTypes::Misc, to_data_category);

        let owned_cooking = OwnedCookingRecipe {
            recipe_id: id,
            category,
            group: recipe.group,
            ingredient: to_owned_ingredient(recipe.ingredient),
            cooking_time: recipe.cooking_time as i32,
            experience: recipe.experience,
            result: OwnedRecipeResult {
                item_id: result_stack.item.registry_key.to_string(),
                count: result_stack.item_count,
            },
        };

        let dynamic_recipe = match station_type {
            WitCookingType::Smelting => {
                DynamicRecipe::Cooking(OwnedCookingRecipeType::Smelting(owned_cooking))
            }
            WitCookingType::Blasting => {
                DynamicRecipe::Cooking(OwnedCookingRecipeType::Blasting(owned_cooking))
            }
            WitCookingType::Smoking => {
                DynamicRecipe::Cooking(OwnedCookingRecipeType::Smoking(owned_cooking))
            }
            WitCookingType::Campfire => {
                DynamicRecipe::Cooking(OwnedCookingRecipeType::CampfireCooking(owned_cooking))
            }
        };

        let server = self
            .server
            .as_ref()
            .ok_or_else(|| wasmtime::Error::msg("Server not available"))?;
        server.recipe_manager.add_recipe(dynamic_recipe);
        Ok(())
    }

    async fn drop(&mut self, _rep: Resource<WitRecipeManager>) -> wasmtime::Result<()> {
        Ok(())
    }
}

const fn to_data_category(cat: WitRecipeCategory) -> RecipeCategoryTypes {
    match cat {
        WitRecipeCategory::Building => RecipeCategoryTypes::Building,
        WitRecipeCategory::Redstone => RecipeCategoryTypes::Restone,
        WitRecipeCategory::Equipment => RecipeCategoryTypes::Equipment,
        WitRecipeCategory::Misc => RecipeCategoryTypes::Misc,
        WitRecipeCategory::Food => RecipeCategoryTypes::Food,
        WitRecipeCategory::Blocks => RecipeCategoryTypes::Blocks,
    }
}

fn to_owned_ingredient(ing: WitIngredient) -> OwnedRecipeIngredient {
    match ing {
        WitIngredient::Item(id) => OwnedRecipeIngredient::Simple(id),
        WitIngredient::Tag(tag) => OwnedRecipeIngredient::Tagged(tag),
        WitIngredient::OneOf(items) => OwnedRecipeIngredient::OneOf(items),
    }
}
