//! Crafting recipes defined in code, like a mod's `CustomRecipe`.
//!
//! ```ignore
//! struct DyeArmor;
//! impl CraftingHandler for DyeArmor {
//!     fn craft(&self, width: u32, grid: Vec<Option<ItemStack>>) -> Option<ItemStack> { None }
//! }
//! context.register_crafting(DyeArmor);
//! ```

use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};

use crate::wit::pumpkin::plugin::item_stack::ItemStack;

/// Crafts grids that no recipe matches.
pub trait CraftingHandler: Send + Sync + 'static {
    /// The result for `grid` (row by row, `width` slots per row), or `None` when it is not this
    /// handler's recipe. Called only when the grid changes and no other recipe matches.
    fn craft(&self, width: u32, grid: Vec<Option<ItemStack>>) -> Option<ItemStack>;
}

static HANDLERS: Mutex<BTreeMap<u32, Arc<dyn CraftingHandler>>> = Mutex::new(BTreeMap::new());
static NEXT_ID: Mutex<u32> = Mutex::new(0);

pub(crate) fn dispatch(
    handler_id: u32,
    width: u32,
    grid: Vec<Option<ItemStack>>,
) -> Option<ItemStack> {
    let handler = HANDLERS
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .get(&handler_id)
        .cloned();
    handler.and_then(|handler| handler.craft(width, grid))
}

impl crate::Context {
    /// Registers `handler` for crafting grids no recipe matches.
    pub fn register_crafting(&self, handler: impl CraftingHandler) {
        let mut next = NEXT_ID.lock().unwrap_or_else(|e| e.into_inner());
        let id = *next;
        *next += 1;
        HANDLERS
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .insert(id, Arc::new(handler));
        self.register_crafting_handler(id);
    }
}
