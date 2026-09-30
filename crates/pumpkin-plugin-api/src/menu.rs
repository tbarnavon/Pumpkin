//! Menus whose slots the plugin defines: see the `menu` WIT interface.
//!
//! The host keeps the items of the plugin's slots. Give them in [`MenuDefinition::contents`] and
//! push changes with [`update_slot`]; the handler is only called when a player acts.
//!
//! ```ignore
//! struct Chest;
//! impl MenuHandler for Chest {
//!     fn handle(&self, menu_id: u32, player: Player, call: MenuCall) -> MenuReply {
//!         match call {
//!             MenuCall::SetItem(set) => { /* store set.item */ MenuReply::None }
//!             MenuCall::MayPlace(_) | MenuCall::MayPickup(_) => MenuReply::Allowed(true),
//!             _ => MenuReply::None,
//!         }
//!     }
//! }
//! let handler = register_menu_handler(Chest);
//! menu::open(&player, handler, Screen::Generic9x3, MenuDefinition { .. })?;
//! menu::update_slot(handler, menu_id, 0, Some(&stack));
//! ```

use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};

use crate::wit::pumpkin::plugin::player::Player;

pub use crate::wit::pumpkin::plugin::menu::{
    MenuAnchor, MenuCall, MenuDefinition, MenuReply, MenuSlot, QuickMoveStep, SetSlot, SlotItem,
    close, open, update_slot,
};

/// Answers the host's calls about the plugin slots of the menus opened with its id.
pub trait MenuHandler: Send + Sync + 'static {
    /// `menu_id` is the one the menu was opened with; `player` is the one viewing it.
    fn handle(&self, menu_id: u32, player: Player, call: MenuCall) -> MenuReply;
}

static MENU_HANDLERS: Mutex<BTreeMap<u32, Arc<dyn MenuHandler>>> = Mutex::new(BTreeMap::new());
static NEXT_MENU_HANDLER_ID: Mutex<u32> = Mutex::new(0);

/// Stores `handler` and returns the id to open menus with.
pub fn register_menu_handler(handler: impl MenuHandler) -> u32 {
    let mut next = NEXT_MENU_HANDLER_ID
        .lock()
        .unwrap_or_else(|e| e.into_inner());
    let id = *next;
    *next += 1;
    MENU_HANDLERS
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .insert(id, Arc::new(handler));
    id
}

pub(crate) fn dispatch(handler_id: u32, menu_id: u32, player: Player, call: MenuCall) -> MenuReply {
    let handler = MENU_HANDLERS
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .get(&handler_id)
        .cloned();
    match handler {
        Some(handler) => handler.handle(menu_id, player, call),
        None => MenuReply::None,
    }
}
