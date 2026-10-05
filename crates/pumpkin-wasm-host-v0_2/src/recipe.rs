use crate::pumpkin::plugin::recipe::{
    CookingRecipe as WitCookingRecipe, CookingType as WitCookingType,
    CraftingRecipeInfo as WitCraftingRecipeInfo, Host as RecipeHost, HostRecipeManager,
    Ingredient as WitIngredient, RecipeCategory as WitRecipeCategory,
    RecipeManager as WitRecipeManager, ShapedRecipe as WitShapedRecipe,
    ShapelessRecipe as WitShapelessRecipe,
};
use pumpkin_data::recipes::RecipeCategoryTypes;
use pumpkin_protocol::codec::recipe::{
    DynamicRecipe, OwnedCookingRecipe, OwnedCookingRecipeType, OwnedCraftingRecipe,
    OwnedRecipeIngredient, OwnedRecipeResult,
};
use pumpkin_wasm_host_common::state::PluginHostState;
use wasmtime::component::Resource;

impl RecipeHost for PluginHostState {}

/// A plugin's `handle-crafting`, as a special crafting handler.
pub struct PluginCraftingHandler {
    pub plugin: std::sync::Arc<pumpkin_wasm_host_common::plugin::WasmPlugin>,
    pub handler_id: u32,
    pub server: std::sync::Weak<pumpkin_core::server::Server>,
}

impl pumpkin_core::server::recipe::SpecialCraftingHandler for PluginCraftingHandler {
    fn craft(
        &self,
        width: usize,
        grid: &[pumpkin_data::item_stack::ItemStack],
    ) -> Option<pumpkin_data::item_stack::ItemStack> {
        let server = self.server.upgrade()?;
        let plugin = self.plugin.clone();
        let handler_id = self.handler_id;
        let grid = grid.to_vec();
        let run = async move {
            let generation = plugin.current();
            let function = generation
                .instance::<crate::Plugin>()
                .func_handle_crafting();
            generation
                .store
                .call_guest(move |mut guest| {
                    Box::pin(async move {
                        let resources = guest.with(|mut store| {
                            grid.into_iter()
                                .map(|stack| {
                                    (!stack.is_empty())
                                        .then(|| {
                                            store.data_mut().add::<super::pumpkin::plugin::item_stack::ItemStack>(
                                                std::sync::Arc::new(tokio::sync::Mutex::new(stack)),
                                            )
                                        })
                                        .transpose()
                                })
                                .collect::<wasmtime::Result<Vec<_>>>()
                        })?;
                        let result = guest
                            .call(function, (handler_id, width as u32, resources))
                            .await?
                            .0;
                        let Some(result) = result else {
                            return Ok(None);
                        };
                        let handle = guest.with(|mut store| store.data_mut().take(result))?;
                        let stack = handle.lock().await.clone();
                        Ok::<_, wasmtime::Error>(Some(stack))
                    })
                })
                .await
        };
        let result = if tokio::runtime::Handle::try_current().is_ok() {
            tokio::task::block_in_place(|| server.runtime.block_on(run))
        } else {
            server.runtime.block_on(run)
        };
        match result {
            Ok(stack) => stack.filter(|stack| !stack.is_empty()),
            Err(error) => {
                tracing::error!(handler_id, error = ?error, "Wasm crafting handler failed");
                None
            }
        }
    }

    fn same_as(&self, other: &dyn std::any::Any) -> bool {
        other.downcast_ref::<Self>().is_some_and(|other| {
            std::sync::Arc::ptr_eq(&self.plugin, &other.plugin)
                && self.handler_id == other.handler_id
        })
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

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

/// Every item id `accepts` takes, for `crafting-recipes-for`.
fn accepted_items(accepts: impl Fn(&pumpkin_data::item::Item) -> bool) -> Vec<String> {
    (1..pumpkin_data::item::Item::count())
        .filter_map(pumpkin_data::item::Item::from_id)
        .filter(|item| accepts(item))
        .map(|item| item.namespaced_name().into_owned())
        .collect()
}

/// A shaped recipe's cells row by row, `None` for an empty one.
fn shaped_cells<T>(
    pattern: &[impl AsRef<str>],
    key: impl Fn(char) -> Option<T>,
) -> (u32, u32, Vec<Option<T>>) {
    let width = pattern
        .iter()
        .map(|row| row.as_ref().chars().count())
        .max()
        .unwrap_or(0);
    let mut cells = Vec::with_capacity(width * pattern.len());
    for row in pattern {
        let mut chars = row.as_ref().chars();
        for _ in 0..width {
            cells.push(chars.next().filter(|c| *c != ' ').and_then(&key));
        }
    }
    (width as u32, pattern.len() as u32, cells)
}

/// The crafting recipes whose result is `item`, as `crafting-recipes-for` lists them.
fn crafting_recipes_for(
    item: &str,
    dynamic: &[pumpkin_protocol::codec::recipe::DynamicRecipe],
) -> Vec<WitCraftingRecipeInfo> {
    use pumpkin_data::recipes::{CraftingRecipeTypes, RECIPES_CRAFTING};
    let wanted = if item.contains(':') {
        item.to_string()
    } else {
        format!("minecraft:{item}")
    };
    let is_wanted = |id: &str| {
        if id.contains(':') {
            id == wanted
        } else {
            wanted.strip_prefix("minecraft:") == Some(id)
        }
    };
    let info =
        |shaped, (width, height, ingredients), result: &str, result_count| WitCraftingRecipeInfo {
            shaped,
            width,
            height,
            ingredients,
            output: result.to_string(),
            output_count: result_count,
        };
    let mut out = Vec::new();
    for recipe in RECIPES_CRAFTING {
        match recipe {
            CraftingRecipeTypes::CraftingShaped {
                key,
                pattern,
                result,
                ..
            } if is_wanted(result.id) => {
                let cells = shaped_cells(pattern, |c| {
                    let (_, ingredient) = key.iter().find(|(k, _)| *k == c)?;
                    Some(accepted_items(|item| ingredient.match_item(item)))
                });
                out.push(info(true, cells, &wanted, result.count));
            }
            CraftingRecipeTypes::CraftingShapeless {
                ingredients,
                result,
                ..
            } if is_wanted(result.id) => {
                let ingredients = ingredients
                    .iter()
                    .map(|ingredient| Some(accepted_items(|item| ingredient.match_item(item))))
                    .collect();
                out.push(info(false, (0, 0, ingredients), &wanted, result.count));
            }
            CraftingRecipeTypes::CraftingTransmute {
                input,
                material,
                result,
                ..
            } if is_wanted(result.id) => {
                let ingredients = [input, material]
                    .iter()
                    .map(|ingredient| Some(accepted_items(|item| ingredient.match_item(item))))
                    .collect();
                out.push(info(false, (0, 0, ingredients), &wanted, result.count));
            }
            _ => {}
        }
    }
    for recipe in dynamic {
        let pumpkin_protocol::codec::recipe::DynamicRecipe::Crafting(recipe) = recipe else {
            continue;
        };
        match recipe {
            OwnedCraftingRecipe::Shaped {
                key,
                pattern,
                result,
                ..
            } if is_wanted(&result.item_id) => {
                let cells = shaped_cells(pattern, |c| {
                    let (_, ingredient) = key.iter().find(|(k, _)| *k == c)?;
                    Some(accepted_items(|item| ingredient.match_item(item)))
                });
                out.push(info(true, cells, &wanted, result.count));
            }
            OwnedCraftingRecipe::Shapeless {
                ingredients,
                result,
                ..
            } if is_wanted(&result.item_id) => {
                let ingredients = ingredients
                    .iter()
                    .map(|ingredient| Some(accepted_items(|item| ingredient.match_item(item))))
                    .collect();
                out.push(info(false, (0, 0, ingredients), &wanted, result.count));
            }
            _ => {}
        }
    }
    out
}

impl HostRecipeManager for PluginHostState {
    fn crafting_recipes_for(
        &mut self,
        _res: Resource<WitRecipeManager>,
        item: String,
    ) -> wasmtime::Result<Vec<WitCraftingRecipeInfo>> {
        let server = self
            .server
            .clone()
            .ok_or_else(|| wasmtime::Error::msg("Server not available"))?;
        Ok(crafting_recipes_for(
            &item,
            &server.recipe_manager.get_dynamic_recipes_internal(),
        ))
    }

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

    fn drop(&mut self, _rep: Resource<WitRecipeManager>) -> wasmtime::Result<()> {
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
