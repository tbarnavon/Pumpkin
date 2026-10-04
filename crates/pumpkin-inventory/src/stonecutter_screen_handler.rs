use std::any::Any;
use std::sync::Arc;
use std::sync::atomic::{AtomicU8, AtomicU16, Ordering};

use crate::player::player_inventory::PlayerInventory;
use crate::screen_handler::{InventoryPlayer, ScreenHandler, ScreenHandlerBehaviour};
use crate::slot::{NormalSlot, Slot};

use crate::inventory::Inventory;
use crate::inventory::SimpleInventory;
use pumpkin_data::data_component_impl::ItemNameImpl;
use pumpkin_data::item::Item;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_data::recipes::{RECIPES_STONECUTTING, StonecutterRecipe};
use pumpkin_data::screen::WindowType;
use pumpkin_data::statistic::StatisticCategory;
use pumpkin_protocol::java::server::play::SlotActionType;

pub struct StonecutterScreenHandler {
    behaviour: ScreenHandlerBehaviour,
    pub input_inventory: Arc<SimpleInventory>,
    pub output_inventory: Arc<SimpleInventory>,
    pub selected_recipe: AtomicU8,
    /// The input item the recipe list was built for; `u16::MAX` when there is none.
    input_item: AtomicU16,
}

impl StonecutterScreenHandler {
    pub fn new(sync_id: u8, player_inventory: &Arc<PlayerInventory>) -> Self {
        let behaviour = ScreenHandlerBehaviour::new(sync_id, Some(WindowType::Stonecutter));
        let input_inventory = Arc::new(SimpleInventory::new(1));
        let output_inventory = Arc::new(SimpleInventory::new(1));

        let mut handler = Self {
            behaviour,
            input_inventory: input_inventory.clone(),
            output_inventory: output_inventory.clone(),
            selected_recipe: AtomicU8::new(u8::MAX),
            input_item: AtomicU16::new(u16::MAX),
        };

        handler.add_slot(Arc::new(NormalSlot::new(
            input_inventory.clone() as Arc<dyn Inventory>,
            0,
        )));
        handler.add_slot(Arc::new(StonecutterOutputSlot::new(
            output_inventory as Arc<dyn Inventory>,
            input_inventory as Arc<dyn Inventory>,
            0,
        )));

        let player_inventory: Arc<dyn Inventory> = player_inventory.clone();

        handler.add_player_slots(&player_inventory);

        handler
    }

    fn update_output(&self) {
        let input_lock = self.input_inventory.get_stack(0);

        if input_lock.is_empty() {
            self.output_inventory.set_stack(0, ItemStack::EMPTY.clone());
            self.selected_recipe.store(u8::MAX, Ordering::Relaxed);
            self.input_item.store(u16::MAX, Ordering::Relaxed);
            return;
        }
        // Vanilla's `slotsChanged`: a different input item clears the selection.
        if self.input_item.swap(input_lock.item.id, Ordering::Relaxed) != input_lock.item.id {
            self.selected_recipe.store(u8::MAX, Ordering::Relaxed);
        }

        let available_recipes = Self::get_available_recipes(&input_lock);
        let recipe_index = self.selected_recipe.load(Ordering::Relaxed);

        if recipe_index != u8::MAX && (recipe_index as usize) < available_recipes.len() {
            let recipe = available_recipes[recipe_index as usize];
            let item = Item::from_registry_key(recipe.result.id).unwrap_or(&Item::AIR);
            let result = ItemStack::new(recipe.result.count, item);
            self.output_inventory.set_stack(0, result);
        } else {
            self.output_inventory.set_stack(0, ItemStack::EMPTY.clone());
        }
    }

    fn get_available_recipes(input: &ItemStack) -> Vec<&'static StonecutterRecipe> {
        let item = input.item;
        let mut recipes: Vec<_> = RECIPES_STONECUTTING
            .iter()
            .filter(|r| r.ingredient.match_item(item))
            .collect();
        // 1.21.1's `RecipeManager.getRecipesFor` sorts by the result's description id, and the
        // client picks a recipe by its index in that order.
        recipes.sort_by_cached_key(|r| {
            Item::from_registry_key(r.result.id)
                .and_then(|item| {
                    ItemStack::new(1, item)
                        .get_data_component::<ItemNameImpl>()
                        .map(|name| name.name.to_string())
                })
                .unwrap_or_default()
        });
        recipes
    }
}

impl ScreenHandler for StonecutterScreenHandler {
    fn get_behaviour(&self) -> &ScreenHandlerBehaviour {
        &self.behaviour
    }

    fn get_behaviour_mut(&mut self) -> &mut ScreenHandlerBehaviour {
        &mut self.behaviour
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn on_button_click(&mut self, _player: &dyn InventoryPlayer, id: i32) -> bool {
        // Vanilla's `clickMenuButton`: select the recipe at that index of the sorted list.
        let input = self.input_inventory.get_stack(0);
        if !input.is_empty()
            && let Ok(index) = u8::try_from(id)
            && usize::from(index) < Self::get_available_recipes(&input).len()
        {
            // Record the current input first, so its change check can't clear the new selection.
            self.update_output();
            self.selected_recipe.store(index, Ordering::Relaxed);
            self.update_output();
            self.send_content_updates();
        }
        true
    }

    fn on_slot_click(
        &mut self,
        slot_index: i32,
        button: i32,
        action_type: SlotActionType,
        player: &dyn InventoryPlayer,
    ) {
        self.internal_on_slot_click(slot_index, button, action_type, player);
        // Taking the result uses up input, and vanilla refills the result from what is left.
        if slot_index == 0 || slot_index == 1 {
            self.update_output();
        }
    }

    fn quick_move(&mut self, player: &dyn InventoryPlayer, slot_index: i32) -> ItemStack {
        let mut stack = ItemStack::EMPTY.clone();
        let slot = self.get_behaviour().slots.get(slot_index as usize).cloned();

        if let Some(slot) = slot {
            let mut slot_stack = slot.get_cloned_stack();
            if !slot_stack.is_empty() {
                stack = slot_stack.clone();
                if slot_index < 2 {
                    // From Stonecutter to Player
                    if !self.insert_item(&mut slot_stack, 2, 38, true) {
                        return ItemStack::EMPTY.clone();
                    }
                    slot.on_quick_move_crafted(slot_stack.clone(), stack.clone());
                } else {
                    // From Player to Stonecutter
                    // Try input slot (0)
                    if !self.insert_item(&mut slot_stack, 0, 1, false) {
                        return ItemStack::EMPTY.clone();
                    }
                }

                if slot_stack.is_empty() {
                    slot.set_stack(ItemStack::EMPTY.clone());
                } else {
                    slot.set_stack(slot_stack.clone());
                }

                if slot_index == 1 {
                    let mut taken_stack = stack.clone();
                    taken_stack.set_count(stack.item_count - slot_stack.item_count);
                    slot.on_take_item(player, &taken_stack);
                    self.update_output();
                }
            }
        }
        stack
    }
}

pub struct StonecutterOutputSlot {
    pub inventory: Arc<dyn Inventory>,
    pub input_inventory: Arc<dyn Inventory>,
    pub index: usize,
    pub id: AtomicU8,
}

impl StonecutterOutputSlot {
    pub fn new(
        inventory: Arc<dyn Inventory>,
        input_inventory: Arc<dyn Inventory>,
        index: usize,
    ) -> Self {
        Self {
            inventory,
            input_inventory,
            index,
            id: AtomicU8::new(0),
        }
    }
}

impl Slot for StonecutterOutputSlot {
    fn get_inventory(&self) -> Arc<dyn Inventory> {
        self.inventory.clone()
    }

    fn get_index(&self) -> usize {
        self.index
    }

    fn set_id(&self, id: usize) {
        self.id.store(id as u8, Ordering::Relaxed);
    }

    fn on_take_item(&self, player: &dyn InventoryPlayer, stack: &ItemStack) {
        player.increment_stat(
            StatisticCategory::Crafted,
            stack.item.id as i32,
            stack.item_count as i32,
        );
        self.input_inventory.remove_stack_specific(0, 1);
        self.mark_dirty();
    }

    fn can_insert(&self, _stack: &ItemStack) -> bool {
        false
    }

    fn get_stack(&self) -> ItemStack {
        self.inventory.get_stack(self.index)
    }

    fn get_cloned_stack(&self) -> ItemStack {
        self.inventory.get_stack(self.index)
    }

    fn has_stack(&self) -> bool {
        !self.inventory.get_stack(self.index).is_empty()
    }

    fn set_stack(&self, stack: ItemStack) {
        self.inventory.set_stack(self.index, stack);
    }

    fn set_stack_prev(&self, _stack: ItemStack, _previous_stack: ItemStack) {
        // Do nothing
    }

    fn mark_dirty(&self) {
        self.inventory.mark_dirty();
    }
}
