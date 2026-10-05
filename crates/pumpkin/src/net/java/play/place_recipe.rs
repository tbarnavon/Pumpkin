#[allow(clippy::wildcard_imports)]
use super::*;

impl JavaClient {
    #[allow(clippy::too_many_lines)]
    pub fn handle_place_recipe(
        &self,
        server: &Arc<Server>,
        player: &Arc<Player>,
        packet: &SPlaceRecipe,
    ) {
        use crate::net::java::recipe_helper::{
            GenericIngredient, compute_biggest_craftable, take_n_ingredient,
        };
        use crate::server::recipe::DynamicRecipe;
        use pumpkin_data::recipes::{CraftingRecipeTypes, RECIPES_COOKING, RECIPES_CRAFTING};
        use pumpkin_data::screen::WindowType;
        use pumpkin_inventory::crafting::recipe_provider::RecipeProvider;

        let target_id = packet.recipe_display_id.0 as usize;
        let use_max = packet.use_max_items;

        let mut click_event = crate::plugin::api::events::player::player_recipe_book_click::PlayerRecipeBookClickEvent::new(
            player.clone(),
            format!("display_{}", packet.recipe_display_id.0),
            use_max,
        );
        server
            .plugin_manager
            .fire_blocking(server, &mut click_event);
        if click_event.cancelled {
            return;
        }

        // Count crafting display IDs.
        let crafting_display_count = RECIPES_CRAFTING
            .iter()
            .filter(|r| {
                !matches!(
                    r,
                    CraftingRecipeTypes::CraftingSpecial
                        | CraftingRecipeTypes::CraftingDecoratedPot { .. }
                )
            })
            .count();
        let cooking_display_count = RECIPES_COOKING.len();
        let dynamic_recipes = server.recipe_manager.get_dynamic_recipes();

        if (crafting_display_count..crafting_display_count + cooking_display_count)
            .contains(&target_id)
        {
            Self::place_cooking_recipe(
                player,
                target_id,
                &RECIPES_COOKING[target_id - crafting_display_count],
                use_max,
            );
            return;
        }

        let (grid_width, crafting_inv) = {
            let screen_handler_arc = player
                .current_screen_handler
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .clone();
            let handler = screen_handler_arc
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            let grid_width: usize = match handler.window_type() {
                Some(WindowType::Crafting) => 3,
                None => 2, // player inventory 2x2
                _ => return,
            };
            (grid_width, handler.get_behaviour().slots[1].get_inventory())
        };

        let grid_size = grid_width * grid_width;
        let mut ingredient_slots: Vec<Option<GenericIngredient<'_>>> = vec![None; grid_size];

        if target_id < crafting_display_count {
            // Crafting recipe
            let mut counter = 0usize;
            let recipe = RECIPES_CRAFTING.iter().find(|r| {
                if matches!(
                    r,
                    CraftingRecipeTypes::CraftingSpecial
                        | CraftingRecipeTypes::CraftingDecoratedPot { .. }
                ) {
                    return false;
                }
                let found = counter == target_id;
                counter += 1;
                found
            });
            let Some(recipe) = recipe else { return };

            match recipe {
                CraftingRecipeTypes::CraftingShaped { pattern, key, .. } => {
                    for (row, row_str) in pattern.iter().enumerate() {
                        for (col, ch) in row_str.chars().enumerate() {
                            if ch != ' '
                                && let Some(ing) =
                                    key.iter().find_map(|(k, v)| (*k == ch).then_some(v))
                                && row * grid_width + col < grid_size
                            {
                                ingredient_slots[row * grid_width + col] =
                                    Some(GenericIngredient::Vanilla(ing));
                            }
                        }
                    }
                }
                CraftingRecipeTypes::CraftingShapeless { ingredients, .. } => {
                    for (i, ing) in ingredients.iter().enumerate().take(grid_size) {
                        ingredient_slots[i] = Some(GenericIngredient::Vanilla(ing));
                    }
                }
                CraftingRecipeTypes::CraftingTransmute {
                    input, material, ..
                } => {
                    if grid_size >= 2 {
                        ingredient_slots[0] = Some(GenericIngredient::Vanilla(input));
                        ingredient_slots[1] = Some(GenericIngredient::Vanilla(material));
                    }
                }
                _ => return,
            }
        } else {
            let dynamic_id = target_id - crafting_display_count - cooking_display_count;
            let Some(DynamicRecipe::Crafting(crafting)) = dynamic_recipes.get(dynamic_id) else {
                return;
            };

            match crafting {
                pumpkin_protocol::codec::recipe::OwnedCraftingRecipe::Shaped {
                    pattern,
                    key,
                    ..
                } => {
                    for (row, row_str) in pattern.iter().enumerate() {
                        for (col, ch) in row_str.chars().enumerate() {
                            if ch != ' '
                                && let Some((_, ing)) = key.iter().find(|(k, _)| *k == ch)
                                && row * grid_width + col < grid_size
                            {
                                ingredient_slots[row * grid_width + col] =
                                    Some(GenericIngredient::Dynamic(ing));
                            }
                        }
                    }
                }

                pumpkin_protocol::codec::recipe::OwnedCraftingRecipe::Shapeless {
                    ingredients,
                    ..
                } => {
                    for (i, ing) in ingredients.iter().enumerate().take(grid_size) {
                        ingredient_slots[i] = Some(GenericIngredient::Dynamic(ing));
                    }
                }
            }
        }

        // Check if this exact recipe is already placed (determines stacking vs fresh fill).
        let recipe_matches = {
            let mut ok = true;
            for (idx, ing) in ingredient_slots.iter().enumerate() {
                let stack = crafting_inv.get_stack(idx);
                match ing {
                    None => {
                        if !stack.is_empty() {
                            ok = false;
                            break;
                        }
                    }
                    Some(ingredient) => {
                        if stack.is_empty() || !ingredient.match_item(stack.item) {
                            ok = false;
                            break;
                        }
                    }
                }
            }
            ok
        };

        // Read minimum count from occupied slots before clearing (needed for stacking).
        let current_min = if recipe_matches && !use_max {
            let mut min = u8::MAX;
            for (idx, ing) in ingredient_slots.iter().enumerate() {
                if ing.is_some() {
                    let stack = crafting_inv.get_stack(idx);
                    if !stack.is_empty() {
                        min = min.min(stack.item_count);
                    }
                }
            }
            if min == u8::MAX { 0 } else { min }
        } else {
            0
        };

        // Always clear the grid first, returning items to inventory.
        for i in 0..grid_size {
            let stack = crafting_inv.remove_stack(i);
            if !stack.is_empty() {
                player.inventory.offer(stack, false, player.as_ref());
            }
        }

        // Determine how many of each ingredient to place per slot.
        let active_ingredients: Vec<GenericIngredient<'_>> =
            ingredient_slots.iter().flatten().copied().collect();
        let amount_to_craft = if use_max {
            compute_biggest_craftable(&active_ingredients, &player.inventory)
        } else if recipe_matches {
            current_min.saturating_add(1)
        } else {
            1
        };

        if amount_to_craft == 0 {
            Self::send_ghost_recipe(player, target_id);
            let screen_handler_arc = player
                .current_screen_handler
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .clone();
            screen_handler_arc
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .send_content_updates();
            return;
        }

        // Fill each grid slot with exactly `amount_to_craft` matching items.
        for (idx, ing) in ingredient_slots.iter().enumerate() {
            let Some(ingredient) = ing else { continue };
            let taken = take_n_ingredient(&player.inventory, ingredient, amount_to_craft);
            if !taken.is_empty() {
                crafting_inv.set_stack(idx, taken);
            }
        }

        let screen_handler_arc = player
            .current_screen_handler
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clone();
        screen_handler_arc
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .send_content_updates();
    }

    /// Places a cooking recipe in a furnace, smoker or blast furnace: vanilla's
    /// `ServerPlaceRecipe` with the input slot as a 1x1 grid.
    fn place_cooking_recipe(
        player: &Arc<Player>,
        display_id: usize,
        recipe: &pumpkin_data::recipes::CookingRecipeType,
        use_max: bool,
    ) {
        use crate::net::java::recipe_helper::{
            GenericIngredient, compute_biggest_craftable, take_n_ingredient,
        };
        use pumpkin_data::recipes::CookingRecipeType;
        use pumpkin_data::screen::WindowType;

        let screen_handler_arc = player
            .current_screen_handler
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clone();
        let mut handler = screen_handler_arc
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let ((Some(WindowType::Furnace), CookingRecipeType::Smelting(cooking))
        | (Some(WindowType::BlastFurnace), CookingRecipeType::Blasting(cooking))
        | (Some(WindowType::Smoker), CookingRecipeType::Smoking(cooking))) =
            (handler.window_type(), recipe)
        else {
            return;
        };
        let input = handler.get_behaviour().slots[0].get_inventory();
        let ingredient = GenericIngredient::Vanilla(&cooking.ingredient);

        // The input goes back to the inventory first; a matching input counts towards what can
        // be placed, and a click without `use_max` adds one more.
        let current = input.get_stack(0);
        let recipe_matches = !current.is_empty() && ingredient.match_item(current.item);
        let current_count = if recipe_matches {
            current.item_count
        } else {
            0
        };
        let removed = input.remove_stack(0);
        if !removed.is_empty() {
            player.inventory.offer(removed, false, player.as_ref());
        }

        let available = compute_biggest_craftable(&[ingredient], &player.inventory);
        if available == 0 {
            drop(handler);
            Self::send_ghost_recipe(player, display_id);
            screen_handler_arc
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .send_content_updates();
            return;
        }
        let amount = if use_max {
            available
        } else if recipe_matches {
            current_count.saturating_add(1).min(available)
        } else {
            1
        };
        let taken = take_n_ingredient(&player.inventory, &ingredient, amount);
        if !taken.is_empty() {
            input.set_stack(0, taken);
        }
        handler.send_content_updates();
    }

    /// Shows the recipe's ingredients as ghost items when the player lacks them
    /// (`ClientboundPlaceGhostRecipePacket`). Clients from 1.21.2 on get the recipe display in
    /// that packet, which isn't sent yet.
    fn send_ghost_recipe(player: &Arc<Player>, display_id: usize) {
        if pumpkin_data::packet::CURRENT_MC_VERSION
            >= pumpkin_util::version::JavaMinecraftVersion::V_1_21_2
        {
            return;
        }
        let Some(recipe_id) =
            pumpkin_protocol::java::server::play::recipe_id_of_display(display_id)
        else {
            return;
        };
        let sync_id = player
            .current_screen_handler
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .sync_id();
        player.try_send_client_packet(
            &pumpkin_protocol::java::client::play::CPlaceGhostRecipe::new(sync_id, &recipe_id),
        );
    }
}
