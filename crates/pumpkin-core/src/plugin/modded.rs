//! What the server calls on plugin-backed blocks, items and per-tick hooks.
//!
//! The behaviour itself lives in the Wasm host, which depends on this crate; these traits are the
//! part the server needs to hold and call it (block and item registries, world tick queues).

use std::any::Any;
use std::sync::Arc;

use pumpkin_data::item_stack::ItemStack;
use pumpkin_data::{BlockDirection, BlockStateId};
use pumpkin_util::Hand;
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector3::Vector3;

use crate::block::BlockBehaviour;
use crate::entity::EntityBase;
use crate::entity::player::Player;
use crate::item::ItemBehaviour;
use crate::server::Server;
use crate::world::World;

/// Whether any plugin block asked for the `ticker` hook, so worlds only look for plugin block
/// entities to tick when one did.
pub static ANY_TICKER: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

/// A plugin's hooks for a modded block: [`BlockBehaviour`] plus the hooks with no
/// `BlockBehaviour` method.
pub trait PluginBlockHooks: BlockBehaviour {
    /// Whether `other` is the same plugin's handler, as registered again after a restart.
    fn is_same_handler(&self, other: &dyn PluginBlockHooks) -> bool;

    /// Whether the plugin asked for the `ticker` hook.
    fn ticks_block_entities(&self) -> bool;

    /// Queues the block entity ticker, for a block that opted in with `ticker`.
    fn block_entity_tick(&self, world: &World, pos: BlockPos, state: BlockStateId);

    /// `Block.attack`. Returns `true` when the plugin handled the click; the caller then keeps a
    /// creative player from breaking the block.
    fn attack(
        &self,
        server: &Server,
        world: &Arc<World>,
        pos: BlockPos,
        player: &Arc<Player>,
        face: BlockDirection,
        location: Vector3<f64>,
    ) -> bool;

    /// `Block.getDrops`, or `None` to use the loot table.
    fn drops(
        &self,
        server: &Server,
        world: &Arc<World>,
        pos: BlockPos,
        state: BlockStateId,
        player: Option<Arc<Player>>,
        tool: Option<ItemStack>,
    ) -> Option<Vec<ItemStack>>;

    /// `BlockEntity.preRemoveSideEffects`.
    fn removed(&self, server: &Server, world: &Arc<World>, pos: BlockPos, state: BlockStateId);

    fn as_any(&self) -> &dyn Any;
}

/// A plugin's hooks for a modded item that the server calls outside [`ItemBehaviour`], reached
/// with [`ItemBehaviour::plugin_hooks`].
pub trait PluginItemHooks: Send + Sync {
    /// Whether both are the same plugin's handler, as registered again after a restart.
    fn same_handler(&self, other: &dyn ItemBehaviour) -> bool;

    /// Whether the plugin asked for `inventory-tick` calls.
    fn ticks_in_inventory(&self) -> bool;

    /// `Item.inventoryTick` for a stack, to queue with [`PluginTickQueue::push_inventory`].
    fn inventory_tick(&self, slot: usize, selected: bool, stack: ItemStack) -> QueuedInventoryTick;

    /// Where this item's per-tick hooks are sent.
    fn tick_target(&self) -> Arc<dyn PluginTickTarget>;

    /// `Item.overrideOtherStackedOnMe` (`on_me`, this item is in the slot) or
    /// `Item.overrideStackedOnOther` (this item is carried). The new slot and cursor stacks when
    /// the plugin took the click over.
    #[expect(clippy::too_many_arguments)]
    fn stacked_click(
        &self,
        server: &Server,
        player: Arc<Player>,
        on_me: bool,
        clicked: &ItemStack,
        carried: &ItemStack,
        secondary: bool,
        slot_modifiable: bool,
    ) -> Option<(ItemStack, ItemStack)>;

    /// `Item.onDestroyed`: an item entity of this item is about to be removed after damage.
    fn destroyed(
        &self,
        server: &Server,
        world: Arc<World>,
        entity: Arc<dyn EntityBase>,
        stack: ItemStack,
    );
}

/// A queued `entity-inside` or `step-on` call.
pub struct QueuedContact {
    pub handler_id: u32,
    pub step: bool,
    pub pos: BlockPos,
    pub state: BlockStateId,
    pub entity: Arc<dyn EntityBase>,
}

/// A queued `inventory-tick` call, made by [`PluginItemHooks::inventory_tick`].
pub struct QueuedInventoryTick {
    pub handler_id: u32,
    pub slot: u32,
    pub selected: bool,
    pub stack: ItemStack,
}

/// A queued `use-tick` call.
pub struct QueuedItemUse {
    pub handler_id: u32,
    pub player: Arc<Player>,
    pub hand: Hand,
    pub stack: ItemStack,
    pub remaining_ticks: i32,
}

/// One plugin's per-tick hook calls for one world and tick.
#[derive(Default)]
pub struct QueuedBatch {
    pub inventories: Vec<(Arc<Player>, Vec<QueuedInventoryTick>)>,
    pub item_uses: Vec<QueuedItemUse>,
    pub entity_contacts: Vec<QueuedContact>,
    pub block_entities: Vec<(u32, BlockPos, BlockStateId)>,
}

/// A plugin that takes a world's per-tick hooks as one batch.
pub trait PluginTickTarget: Send + Sync {
    /// Sends the plugin its batch for this tick.
    fn handle_tick_batch(&self, server: &Server, world: &Arc<World>, batch: QueuedBatch);
}

/// A world's per-tick plugin hooks, sent as one `handle-tick-batch` call per plugin.
///
/// `ticker`, `entity-inside`, `step-on` and `inventory-tick` are queued while the world ticks
/// and sent by [`Self::flush`]; a guest call costs ~35 µs of host plumbing, a queued hook far
/// less.
#[derive(Default)]
pub struct PluginTickQueue {
    batches: std::sync::Mutex<Vec<(Arc<dyn PluginTickTarget>, QueuedBatch)>>,
}

impl PluginTickQueue {
    /// Adds to `target`'s batch for this tick.
    pub fn push(&self, target: &Arc<dyn PluginTickTarget>, add: impl FnOnce(&mut QueuedBatch)) {
        let mut batches = self
            .batches
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if let Some((_, batch)) = batches
            .iter_mut()
            .find(|(t, _)| std::ptr::addr_eq(Arc::as_ptr(t), Arc::as_ptr(target)))
        {
            add(batch);
        } else {
            let mut batch = QueuedBatch::default();
            add(&mut batch);
            batches.push((target.clone(), batch));
        }
    }

    /// Queues a player's `inventory-tick` calls for one plugin.
    pub fn push_inventory(
        &self,
        target: &Arc<dyn PluginTickTarget>,
        player: Arc<Player>,
        ticks: Vec<QueuedInventoryTick>,
    ) {
        self.push(target, |batch| batch.inventories.push((player, ticks)));
    }

    /// Sends each plugin its batch for this tick, one guest call per plugin.
    pub fn flush(&self, server: &Server, world: &Arc<World>) {
        let batches = std::mem::take(
            &mut *self
                .batches
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner),
        );
        for (target, batch) in batches {
            target.handle_tick_batch(server, world, batch);
        }
    }
}
