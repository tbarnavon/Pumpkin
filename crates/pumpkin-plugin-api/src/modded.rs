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

use crate::wit::pumpkin::plugin::common::BlockPos;
pub use crate::wit::pumpkin::plugin::modded::{
    BlockCall, BlockHit, BlockHooks, BlockReply, Breaking, EntityContact, Interaction,
    InteractionResult, InventoryTick, Placement, Removal, ShapeBox, StreamCodecNode, Tick,
    TickBatch, open_menu, register_component_stream_codec, set_block_collision_shape,
};
pub use crate::wit::pumpkin::plugin::modded::{
    ChannelFlow, Loader, LoaderChannel, register_loader_channels,
};
use crate::wit::pumpkin::plugin::player::Player;
use crate::wit::pumpkin::plugin::server::Server;
use crate::wit::pumpkin::plugin::world::World;

/// Handles the hooks of the blocks it was registered for.
pub trait BlockHookHandler: Send + Sync + 'static {
    /// Called at the hook points in [`BlockHooks`]. Return [`BlockReply::None`] to keep the
    /// host's default for that hook.
    fn handle(&self, server: Server, call: BlockCall) -> BlockReply;

    /// [`BlockHooks::TICKER`]: one block entity, every tick (`EntityBlock.getTicker`).
    fn block_entity_tick(&self, _server: &Server, _world: &World, _pos: BlockPos, _state: u16) {}

    /// [`BlockHooks::ENTITY_INSIDE`] and [`BlockHooks::STEP_ON`] (`contact.step`): an entity
    /// touches the block this tick.
    fn entity_contact(&self, _server: &Server, _world: &World, _contact: EntityContact) {}
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

pub use crate::wit::pumpkin::plugin::modded::{
    ItemCall, ItemDestroyed, ItemHooks, ItemStackedClick, ItemUse, ItemUseOnBlock, ItemUseTick,
    ItemUsing, StackedResult,
};

/// Handles the hooks of the items it was registered for.
pub trait ItemHookHandler: Send + Sync + 'static {
    /// Called at the hook points in [`ItemHooks`]. Reply with [`BlockReply::Interaction`] to
    /// report a result, or [`BlockReply::None`] to pass.
    fn handle(&self, server: Server, call: ItemCall) -> BlockReply;

    /// [`ItemHooks::INVENTORY_TICK`]: one stack in `player`'s inventory, every tick
    /// (`Item.inventoryTick`).
    fn inventory_tick(
        &self,
        _server: &Server,
        _world: &World,
        _player: &Player,
        _tick: InventoryTick,
    ) {
    }

    /// [`ItemHooks::USE_TICK`]: the item `tick.player` is using, every tick (`Item.onUseTick`).
    fn use_tick(&self, _server: &Server, _world: &World, _tick: ItemUseTick) {}
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

/// Runs a tick batch in vanilla's tick order: inventories (players), entity contacts
/// (entities), then block entities.
pub(crate) fn dispatch_tick_batch(server: Server, batch: TickBatch) {
    let blocks = BLOCK_HOOK_HANDLERS
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .clone();
    let items = ITEM_HOOK_HANDLERS
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .clone();
    let world = &batch.world;
    for inventory in batch.inventories {
        for tick in inventory.ticks {
            if let Some(handler) = items.get(&tick.handler_id) {
                handler.inventory_tick(&server, world, &inventory.player, tick);
            }
        }
    }
    for item_use in batch.item_uses {
        if let Some(handler) = items.get(&item_use.handler_id) {
            handler.use_tick(&server, world, item_use);
        }
    }
    for contact in batch.entity_contacts {
        if let Some(handler) = blocks.get(&contact.handler_id) {
            handler.entity_contact(&server, world, contact);
        }
    }
    for tick in batch.block_entities {
        if let Some(handler) = blocks.get(&tick.handler_id) {
            handler.block_entity_tick(&server, world, tick.pos, tick.state);
        }
    }
}
