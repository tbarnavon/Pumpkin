//! Behaviour and storage for modded blocks: see the `modded` WIT interface.
//!
//! ```ignore
//! struct Drawers;
//! impl BlockHookHandler for Drawers {
//!     fn handle(&self, server: Server, call: BlockCall) -> BlockReply {
//!         match call {
//!             BlockCall::Use(interaction) => BlockReply::Interaction(InteractionResult::Success),
//!             _ => BlockReply::None,
//!         }
//!     }
//! }
//! context.register_block_handler(Drawers, &["mymod:drawer"], BlockHooks::USE)?;
//! ```

use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};

pub use crate::wit::pumpkin::plugin::modded::{
    BlockCall, BlockHit, BlockHooks, BlockReply, Breaking, Interaction, InteractionResult,
    Placement, Removal, Tick, get_block_entity_data, open_menu, remove_block_entity_data,
    set_block_entity_data,
};
use crate::wit::pumpkin::plugin::server::Server;

/// Handles the hooks of the blocks it was registered for.
pub trait BlockHookHandler: Send + Sync + 'static {
    /// Called at the hook points in [`BlockHooks`]. Return [`BlockReply::None`] to keep the
    /// host's default for that hook.
    fn handle(&self, server: Server, call: BlockCall) -> BlockReply;
}

pub(crate) static BLOCK_HOOK_HANDLERS: Mutex<BTreeMap<u32, Arc<dyn BlockHookHandler>>> =
    Mutex::new(BTreeMap::new());
static NEXT_HANDLER_ID: Mutex<u32> = Mutex::new(0);

/// Stores `handler` and returns the id the host will call it with.
pub(crate) fn register_handler(handler: impl BlockHookHandler) -> u32 {
    let mut next = NEXT_HANDLER_ID.lock().unwrap_or_else(|e| e.into_inner());
    let id = *next;
    *next += 1;
    BLOCK_HOOK_HANDLERS
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .insert(id, Arc::new(handler));
    id
}

pub(crate) fn dispatch(handler_id: u32, server: Server, call: BlockCall) -> BlockReply {
    let handler = BLOCK_HOOK_HANDLERS
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .get(&handler_id)
        .cloned();
    match handler {
        Some(handler) => handler.handle(server, call),
        None => BlockReply::None,
    }
}

impl crate::Context {
    /// Registers `handler` for the given namespaced blocks. The host calls it for each hook in
    /// `hooks`.
    pub fn register_block_handler(
        &self,
        handler: impl BlockHookHandler,
        blocks: &[&str],
        hooks: BlockHooks,
    ) -> crate::Result<()> {
        let id = register_handler(handler);
        let blocks: Vec<String> = blocks.iter().map(|b| (*b).to_string()).collect();
        self.register_block_hooks(id, &blocks, hooks)
    }
}

pub use crate::wit::pumpkin::plugin::modded::{ItemCall, ItemHooks, ItemUse, ItemUseOnBlock};

/// Handles the hooks of the items it was registered for.
pub trait ItemHookHandler: Send + Sync + 'static {
    /// Called at the hook points in [`ItemHooks`]. Reply with [`BlockReply::Interaction`] to
    /// report a result, or [`BlockReply::None`] to pass.
    fn handle(&self, server: Server, call: ItemCall) -> BlockReply;
}

pub(crate) static ITEM_HOOK_HANDLERS: Mutex<BTreeMap<u32, Arc<dyn ItemHookHandler>>> =
    Mutex::new(BTreeMap::new());

pub(crate) fn dispatch_item(handler_id: u32, server: Server, call: ItemCall) -> BlockReply {
    let handler = ITEM_HOOK_HANDLERS
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .get(&handler_id)
        .cloned();
    match handler {
        Some(handler) => handler.handle(server, call),
        None => BlockReply::None,
    }
}

impl crate::Context {
    /// Registers `handler` for the given namespaced items. The host calls it for each hook in
    /// `hooks`.
    pub fn register_item_handler(
        &self,
        handler: impl ItemHookHandler,
        items: &[&str],
        hooks: ItemHooks,
    ) -> crate::Result<()> {
        let mut next = NEXT_HANDLER_ID.lock().unwrap_or_else(|e| e.into_inner());
        let id = *next;
        *next += 1;
        drop(next);
        ITEM_HOOK_HANDLERS
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .insert(id, Arc::new(handler));
        let items: Vec<String> = items.iter().map(|i| (*i).to_string()).collect();
        self.register_item_hooks(id, &items, hooks)
    }
}
