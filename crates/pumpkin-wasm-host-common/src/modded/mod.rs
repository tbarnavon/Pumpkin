//! Plugin-backed block and item behaviour, shared by every API version.
//!
//! The server holds these through the traits in `pumpkin_core::plugin::modded`, so there is one
//! of each whatever API a plugin was built against. Calling into the plugin is the only part that
//! depends on the version: each version's host implements [`GuestCalls`], which builds that
//! version's call from the data here and turns its reply back into it.

use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

use pumpkin_data::{Block, BlockDirection, BlockStateId, HorizontalFacingExt};
use pumpkin_util::Hand;
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector3::Vector3;
use tokio::sync::Mutex;

use pumpkin_core::block::registry::BlockActionResult;
use pumpkin_core::block::{
    BlockBehaviour, EmitsRedstonePowerArgs, GetComparatorOutputArgs, GetRedstonePowerArgs,
    GetStateForNeighborUpdateArgs, NormalUseArgs, OnEntityCollisionArgs, OnEntityStepArgs,
    OnNeighborUpdateArgs, OnPlaceArgs, OnScheduledTickArgs, PlayerPlacedArgs, RandomTickArgs,
    UseWithItemArgs,
};
use pumpkin_core::entity::EntityBase;
use pumpkin_core::entity::player::Player;
use pumpkin_core::net::java::loaders::ModLoader;
use pumpkin_core::plugin::modded::{
    PluginBlockHooks, PluginItemHooks, PluginTickTarget, QueuedBatch, QueuedContact,
    QueuedInventoryTick, QueuedItemUse,
};
use pumpkin_core::server::Server;
use pumpkin_core::world::World;

use crate::plugin::{PluginGeneration, WasmPlugin};

pub mod menu;

/// The mod loader the player joined with, if any.
pub fn client_loader(player: &Player) -> Option<ModLoader> {
    match player.client.as_ref() {
        pumpkin_core::net::ClientPlatform::Java(client) => client.mod_loader,
        pumpkin_core::net::ClientPlatform::Bedrock(_) => None,
    }
}

/// What the `modded` API answers while `[modded] enabled` is off in the server config.
pub const DISABLED: &str = "modded content is disabled ([modded] enabled = false)";

bitflags::bitflags! {
    /// The `block-hooks` a plugin asked for, the same in every API version.
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub struct BlockHooks: u32 {
        const PLACEMENT_STATE = 1 << 0;
        const PLACED = 1 << 1;
        const USE = 1 << 2;
        const ATTACK = 1 << 3;
        const DROPS = 1 << 4;
        const REMOVED = 1 << 5;
        const SCHEDULED_TICK = 1 << 6;
        const NEIGHBOR_CHANGED = 1 << 7;
        const RANDOM_TICK = 1 << 8;
        const UPDATE_SHAPE = 1 << 9;
        const SIGNAL_SOURCE = 1 << 10;
        const ANALOG_OUTPUT = 1 << 11;
        const ENTITY_INSIDE = 1 << 12;
        const STEP_ON = 1 << 13;
        const TICKER = 1 << 14;
    }
}

bitflags::bitflags! {
    /// The `item-hooks` a plugin asked for, the same in every API version.
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub struct ItemHooks: u32 {
        const USE_ON_BLOCK = 1 << 0;
        const USE = 1 << 1;
        const INVENTORY_TICK = 1 << 2;
        const STACKED_ON_ME = 1 << 3;
        const STACKED_ON_OTHER = 1 << 4;
        const DESTROYED = 1 << 5;
        const NOT_IN_CONTAINERS = 1 << 6;
        const FINISH_USING = 1 << 7;
        const RELEASE_USING = 1 << 8;
        const USE_TICK = 1 << 9;
        const STOP_USING = 1 << 10;
        const USE_ON_RELEASE = 1 << 11;
    }
}

/// The `interaction-result` of a hook.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InteractionResult {
    Pass,
    Success,
    Consume,
    Fail,
}

/// A block's hooks as implemented by a Wasm plugin.
pub struct PluginBlock {
    pub plugin: Arc<WasmPlugin>,
    pub handler_id: u32,
    pub hooks: BlockHooks,
}

/// Resources a hook call needs, created in the plugin's store right before the call.
pub enum CallData {
    Placement {
        world: Arc<World>,
        pos: BlockPos,
        state: BlockStateId,
        player: Option<Arc<Player>>,
        face: BlockDirection,
        location: Vector3<f64>,
        horizontal_facing: BlockDirection,
        item: Option<Arc<Mutex<pumpkin_data::item_stack::ItemStack>>>,
        placed: bool,
    },
    Interaction {
        world: Arc<World>,
        pos: BlockPos,
        state: BlockStateId,
        player: Arc<Player>,
        hand: Hand,
        face: BlockDirection,
        location: Vector3<f64>,
        attack: bool,
    },
    Breaking {
        world: Arc<World>,
        pos: BlockPos,
        state: BlockStateId,
        player: Option<Arc<Player>>,
        tool: Option<pumpkin_data::item_stack::ItemStack>,
    },
    Removal {
        world: Arc<World>,
        pos: BlockPos,
        state: BlockStateId,
    },
    Tick {
        world: Arc<World>,
        pos: BlockPos,
        state: BlockStateId,
        kind: TickKind,
    },
    ShapeUpdate {
        world: Arc<World>,
        pos: BlockPos,
        state: BlockStateId,
        direction: BlockDirection,
        neighbor_pos: BlockPos,
        neighbor_state: BlockStateId,
    },
    NeighborChanged {
        world: Arc<World>,
        pos: BlockPos,
        state: BlockStateId,
        source: &'static str,
    },
}

#[derive(Clone, Copy)]
pub enum TickKind {
    Scheduled,
    Random,
}

/// A call into the guest, run to completion by the caller.
pub type GuestFuture<T> = Pin<Box<dyn Future<Output = wasmtime::Result<T>>>>;

/// The calls into a plugin that depend on its API version. Each version's host has one, and every
/// [`WasmPlugin`] points at its version's.
pub trait GuestCalls: Send + Sync {
    /// `handle-tick-batch`: a world's queued per-tick hooks.
    fn tick_batch(
        &self,
        generation: Arc<PluginGeneration>,
        world: Arc<World>,
        batch: QueuedBatch,
    ) -> GuestFuture<()>;

    /// `handle-block-hook`.
    fn block_hook(
        &self,
        generation: Arc<PluginGeneration>,
        handler_id: u32,
        data: CallData,
    ) -> GuestFuture<HookReply>;

    /// `handle-item-hook`.
    fn item_hook(
        &self,
        generation: Arc<PluginGeneration>,
        handler_id: u32,
        data: ItemCallData,
    ) -> GuestFuture<ItemReply>;

    /// `handle-menu-call`.
    fn menu_call(
        &self,
        generation: Arc<PluginGeneration>,
        handler_id: u32,
        menu_id: u32,
        viewer: Arc<Player>,
        call: menu::Call,
    ) -> GuestFuture<menu::Reply>;
}

impl PluginTickTarget for WasmPlugin {
    fn handle_tick_batch(&self, server: &Server, world: &Arc<World>, batch: QueuedBatch) {
        let run = self.guest.tick_batch(self.current(), world.clone(), batch);
        if let Err(error) = block_on(server, run) {
            tracing::error!(error = ?error, "Wasm tick batch failed");
        }
    }
}

/// What a hook call returned, with item-stack resources already resolved.
pub enum HookReply {
    None,
    State(u16),
    Interaction(InteractionResult),
    Drops(Vec<pumpkin_data::item_stack::ItemStack>),
}

/// Runs a plugin call to completion from synchronous server code.
fn block_on<T>(server: &Server, run: impl Future<Output = T>) -> T {
    if tokio::runtime::Handle::try_current().is_ok() {
        tokio::task::block_in_place(|| server.runtime.block_on(run))
    } else {
        server.runtime.block_on(run)
    }
}

impl PluginBlock {
    /// Calls the plugin synchronously, the way custom AI goals do (see `mob.rs`).
    fn invoke(&self, server: &Server, data: CallData) -> HookReply {
        let plugin = self.plugin.clone();
        let handler_id = self.handler_id;
        let run = plugin.guest.block_hook(plugin.current(), handler_id, data);

        match block_on(server, run) {
            Ok(reply) => reply,
            Err(error) => {
                tracing::error!(handler_id, error = ?error, "Wasm block hook failed");
                HookReply::None
            }
        }
    }

    const fn interaction(result: InteractionResult) -> BlockActionResult {
        match result {
            InteractionResult::Pass => BlockActionResult::Pass,
            InteractionResult::Success => BlockActionResult::Success,
            InteractionResult::Consume => BlockActionResult::Consume,
            InteractionResult::Fail => BlockActionResult::Fail,
        }
    }

    /// Queues `entity-inside` or `step-on` for this tick's batch.
    fn entity_contact(
        &self,
        world: &World,
        pos: BlockPos,
        state: BlockStateId,
        entity: &dyn EntityBase,
        step: bool,
    ) {
        let Some(handle) = world.get_entity_by_id(entity.get_entity().entity_id) else {
            return;
        };
        let target: Arc<dyn PluginTickTarget> = self.plugin.clone();
        world.plugin_ticks.push(&target, |batch| {
            batch.entity_contacts.push(QueuedContact {
                handler_id: self.handler_id,
                step,
                pos,
                state,
                entity: handle,
            });
        });
    }
}

impl PluginBlockHooks for PluginBlock {
    fn is_same_handler(&self, other: &dyn PluginBlockHooks) -> bool {
        other.as_any().downcast_ref::<Self>().is_some_and(|other| {
            Arc::ptr_eq(&self.plugin, &other.plugin) && self.handler_id == other.handler_id
        })
    }

    fn ticks_block_entities(&self) -> bool {
        self.hooks.contains(BlockHooks::TICKER)
    }

    fn block_entity_tick(&self, world: &World, pos: BlockPos, state: BlockStateId) {
        let target: Arc<dyn PluginTickTarget> = self.plugin.clone();
        world.plugin_ticks.push(&target, |batch| {
            batch.block_entities.push((self.handler_id, pos, state));
        });
    }

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
    ) -> bool {
        if !self.hooks.contains(BlockHooks::ATTACK) {
            return false;
        }
        let state = world.get_block_state_id(&pos);
        let reply = self.invoke(
            server,
            CallData::Interaction {
                world: world.clone(),
                pos,
                state,
                player: player.clone(),
                hand: Hand::Right,
                face,
                location,
                attack: true,
            },
        );
        matches!(reply, HookReply::Interaction(result) if result != InteractionResult::Pass)
    }

    /// `Block.getDrops`, or `None` to use the loot table.
    fn drops(
        &self,
        server: &Server,
        world: &Arc<World>,
        pos: BlockPos,
        state: BlockStateId,
        player: Option<Arc<Player>>,
        tool: Option<pumpkin_data::item_stack::ItemStack>,
    ) -> Option<Vec<pumpkin_data::item_stack::ItemStack>> {
        if !self.hooks.contains(BlockHooks::DROPS) {
            return None;
        }
        match self.invoke(
            server,
            CallData::Breaking {
                world: world.clone(),
                pos,
                state,
                player,
                tool,
            },
        ) {
            HookReply::Drops(drops) => Some(drops),
            _ => None,
        }
    }

    /// `BlockEntity.preRemoveSideEffects`.
    fn removed(&self, server: &Server, world: &Arc<World>, pos: BlockPos, state: BlockStateId) {
        if self.hooks.contains(BlockHooks::REMOVED) {
            self.invoke(
                server,
                CallData::Removal {
                    world: world.clone(),
                    pos,
                    state,
                },
            );
        }
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

/// World position of a hit from the block position and the in-block cursor.
fn hit_location(pos: &BlockPos, cursor: &Vector3<f32>) -> Vector3<f64> {
    Vector3::new(
        f64::from(pos.0.x) + f64::from(cursor.x),
        f64::from(pos.0.y) + f64::from(cursor.y),
        f64::from(pos.0.z) + f64::from(cursor.z),
    )
}

impl BlockBehaviour for PluginBlock {
    fn on_place(&self, args: OnPlaceArgs<'_>) -> BlockStateId {
        let default = args.block.default_state.id;
        if !self.hooks.contains(BlockHooks::PLACEMENT_STATE) {
            return default;
        }
        let world = args.player.world();
        let Some(player) = world.get_player_by_id(args.player.entity_id()) else {
            return default;
        };
        let cursor = Vector3::new(
            args.use_item_on.cursor_pos.x,
            args.use_item_on.cursor_pos.y,
            args.use_item_on.cursor_pos.z,
        );
        let hand = Hand::from_packet_id(args.use_item_on.hand.0).ok();
        let item =
            hand.map(|hand| Arc::new(Mutex::new(player.inventory().get_stack_in_hand(hand))));
        let reply = self.invoke(
            args.server,
            CallData::Placement {
                world,
                pos: *args.position,
                state: default,
                player: Some(player),
                face: args.direction,
                location: hit_location(args.position, &cursor),
                horizontal_facing: args
                    .player
                    .get_entity()
                    .get_horizontal_facing()
                    .to_block_direction(),
                item,
                placed: false,
            },
        );
        match reply {
            HookReply::State(state) => BlockStateId::new(state)
                .filter(|state| Block::from_state_id(*state) == args.block)
                .unwrap_or(default),
            _ => default,
        }
    }

    fn player_placed(&self, args: PlayerPlacedArgs<'_>) {
        if !self.hooks.contains(BlockHooks::PLACED) {
            return;
        }
        let Some(server) = args.world.server.upgrade() else {
            return;
        };
        let Some(player) = args.world.get_player_by_id(args.player.entity_id()) else {
            return;
        };
        // The stack is still in hand here: whichever hand holds this block's item.
        let item = [Hand::Right, Hand::Left]
            .into_iter()
            .map(|hand| player.inventory().get_stack_in_hand(hand))
            .find(|stack| Block::from_item_id(stack.item.id) == Some(args.block))
            .map(|stack| Arc::new(Mutex::new(stack)));
        self.invoke(
            &server,
            CallData::Placement {
                world: args.world.clone(),
                pos: *args.position,
                state: args.state_id,
                player: Some(player),
                face: args.direction,
                location: args.position.to_centered_f64(),
                horizontal_facing: args
                    .player
                    .get_entity()
                    .get_horizontal_facing()
                    .to_block_direction(),
                item,
                placed: true,
            },
        );
    }

    fn normal_use(&self, args: NormalUseArgs<'_>) -> BlockActionResult {
        if !self.hooks.contains(BlockHooks::USE) {
            return BlockActionResult::Pass;
        }
        let Some(server) = args.world.server.upgrade() else {
            return BlockActionResult::Pass;
        };
        let reply = self.invoke(
            &server,
            CallData::Interaction {
                world: args.world.clone(),
                pos: *args.position,
                state: args.world.get_block_state_id(args.position),
                player: args.player.clone(),
                hand: Hand::Right,
                face: *args.hit.face,
                location: hit_location(args.position, args.hit.cursor_pos),
                attack: false,
            },
        );
        match reply {
            HookReply::Interaction(result) => Self::interaction(result),
            _ => BlockActionResult::Pass,
        }
    }

    fn use_with_item(&self, _args: UseWithItemArgs<'_>) -> BlockActionResult {
        // Vanilla `useItemOn` defaults to trying `useWithoutItem`, which is where `use` runs.
        BlockActionResult::PassToDefaultBlockAction
    }

    fn emits_redstone_power(&self, _args: EmitsRedstonePowerArgs<'_>) -> bool {
        self.hooks.contains(BlockHooks::SIGNAL_SOURCE)
    }

    fn get_weak_redstone_power(&self, args: GetRedstonePowerArgs<'_>) -> u8 {
        if !self.hooks.contains(BlockHooks::SIGNAL_SOURCE) {
            return 0;
        }
        args.world
            .plugin_signals(args.position)
            .weak(args.direction)
    }

    fn get_strong_redstone_power(&self, args: GetRedstonePowerArgs<'_>) -> u8 {
        if !self.hooks.contains(BlockHooks::SIGNAL_SOURCE) {
            return 0;
        }
        args.world
            .plugin_signals(args.position)
            .strong(args.direction)
    }

    fn get_comparator_output(&self, args: GetComparatorOutputArgs<'_>) -> Option<u8> {
        self.hooks
            .contains(BlockHooks::ANALOG_OUTPUT)
            .then(|| args.world.plugin_signals(args.position).comparator)
    }

    fn on_entity_collision(&self, args: OnEntityCollisionArgs<'_>) {
        if self.hooks.contains(BlockHooks::ENTITY_INSIDE) {
            self.entity_contact(
                args.world,
                *args.position,
                args.state.id,
                args.entity,
                false,
            );
        }
    }

    fn on_entity_step(&self, args: OnEntityStepArgs<'_>) {
        if self.hooks.contains(BlockHooks::STEP_ON) {
            self.entity_contact(args.world, *args.position, args.state.id, args.entity, true);
        }
    }

    fn random_tick(&self, args: RandomTickArgs<'_>) {
        if !self.hooks.contains(BlockHooks::RANDOM_TICK) {
            return;
        }
        let Some(server) = args.world.server.upgrade() else {
            return;
        };
        self.invoke(
            &server,
            CallData::Tick {
                world: args.world.clone(),
                pos: *args.position,
                state: args.world.get_block_state_id(args.position),
                kind: TickKind::Random,
            },
        );
    }

    fn get_state_for_neighbor_update(
        &self,
        args: GetStateForNeighborUpdateArgs<'_>,
    ) -> BlockStateId {
        if !self.hooks.contains(BlockHooks::UPDATE_SHAPE) {
            return args.state_id;
        }
        let Some(server) = args.world.server.upgrade() else {
            return args.state_id;
        };
        // The hook needs the world as a resource, which takes the shared handle.
        let Some(world) = server
            .worlds
            .load()
            .iter()
            .find(|world| std::ptr::eq(world.as_ref(), args.world))
            .cloned()
        else {
            return args.state_id;
        };
        match self.invoke(
            &server,
            CallData::ShapeUpdate {
                world,
                pos: *args.position,
                state: args.state_id,
                direction: args.direction,
                neighbor_pos: *args.neighbor_position,
                neighbor_state: args.neighbor_state_id,
            },
        ) {
            HookReply::State(state) => BlockStateId::new_or_air(state),
            _ => args.state_id,
        }
    }

    fn on_neighbor_update(&self, args: OnNeighborUpdateArgs<'_>) {
        if !self.hooks.contains(BlockHooks::NEIGHBOR_CHANGED) {
            return;
        }
        let Some(server) = args.world.server.upgrade() else {
            return;
        };
        self.invoke(
            &server,
            CallData::NeighborChanged {
                world: args.world.clone(),
                pos: *args.position,
                state: args.world.get_block_state_id(args.position),
                source: args.source_block.name,
            },
        );
    }

    fn on_scheduled_tick(&self, args: OnScheduledTickArgs<'_>) {
        if !self.hooks.contains(BlockHooks::SCHEDULED_TICK) {
            return;
        }
        let Some(server) = args.world.server.upgrade() else {
            return;
        };
        self.invoke(
            &server,
            CallData::Tick {
                world: args.world.clone(),
                pos: *args.position,
                state: args.world.get_block_state_id(args.position),
                kind: TickKind::Scheduled,
            },
        );
    }
}

/// An item's hooks as implemented by a Wasm plugin.
pub struct PluginItem {
    pub plugin: Arc<WasmPlugin>,
    pub handler_id: u32,
    pub hooks: ItemHooks,
}

pub enum ItemCallData {
    UseOnBlock {
        world: Arc<World>,
        pos: BlockPos,
        state: BlockStateId,
        player: Arc<Player>,
        hand: Hand,
        face: BlockDirection,
        location: Vector3<f64>,
        stack: pumpkin_data::item_stack::ItemStack,
    },
    Use {
        world: Arc<World>,
        player: Arc<Player>,
        hand: Hand,
        stack: pumpkin_data::item_stack::ItemStack,
    },
    Stacked {
        player: Arc<Player>,
        on_me: bool,
        clicked: pumpkin_data::item_stack::ItemStack,
        carried: pumpkin_data::item_stack::ItemStack,
        secondary: bool,
        slot_modifiable: bool,
    },
    Destroyed {
        world: Arc<World>,
        entity: Arc<dyn EntityBase>,
        stack: pumpkin_data::item_stack::ItemStack,
    },
    Using {
        hook: ItemHooks,
        world: Arc<World>,
        player: Arc<Player>,
        hand: Hand,
        stack: pumpkin_data::item_stack::ItemStack,
        remaining_ticks: i32,
    },
}

/// What an item hook call returned, with item-stack resources already resolved.
pub enum ItemReply {
    None,
    Interaction(InteractionResult),
    Stacked(
        pumpkin_data::item_stack::ItemStack,
        pumpkin_data::item_stack::ItemStack,
    ),
    Stack(pumpkin_data::item_stack::ItemStack),
}

impl PluginItem {
    /// Calls `finish-using`, `release-using` or `stop-using` for the player's item in use.
    fn using(
        &self,
        hook: ItemHooks,
        player: &Player,
        hand: Hand,
        stack: &pumpkin_data::item_stack::ItemStack,
        remaining_ticks: i32,
    ) -> ItemReply {
        if !self.hooks.contains(hook) {
            return ItemReply::None;
        }
        let world = player.world();
        let Some(server) = world.server.upgrade() else {
            return ItemReply::None;
        };
        let Some(player) = world.get_player_by_id(player.entity_id()) else {
            return ItemReply::None;
        };
        self.invoke_reply(
            &server,
            ItemCallData::Using {
                hook,
                world,
                player,
                hand,
                stack: stack.clone(),
                remaining_ticks,
            },
        )
    }

    fn invoke(&self, server: &Server, data: ItemCallData) -> Option<InteractionResult> {
        match self.invoke_reply(server, data) {
            ItemReply::Interaction(result) => Some(result),
            _ => None,
        }
    }

    fn invoke_reply(&self, server: &Server, data: ItemCallData) -> ItemReply {
        let plugin = self.plugin.clone();
        let handler_id = self.handler_id;
        let run = plugin.guest.item_hook(plugin.current(), handler_id, data);
        block_on(server, run).unwrap_or_else(|error| {
            tracing::error!(handler_id, error = ?error, "Wasm item hook failed");
            ItemReply::None
        })
    }
}

impl PluginItemHooks for PluginItem {
    fn same_handler(&self, other: &dyn pumpkin_core::item::ItemBehaviour) -> bool {
        other.as_any().downcast_ref::<Self>().is_some_and(|other| {
            Arc::ptr_eq(&self.plugin, &other.plugin) && self.handler_id == other.handler_id
        })
    }

    fn ticks_in_inventory(&self) -> bool {
        self.hooks.contains(ItemHooks::INVENTORY_TICK)
    }

    fn inventory_tick(
        &self,
        slot: usize,
        selected: bool,
        stack: pumpkin_data::item_stack::ItemStack,
    ) -> QueuedInventoryTick {
        QueuedInventoryTick {
            handler_id: self.handler_id,
            slot: slot as u32,
            selected,
            stack,
        }
    }

    fn tick_target(&self) -> Arc<dyn PluginTickTarget> {
        self.plugin.clone()
    }

    fn stacked_click(
        &self,
        server: &Server,
        player: Arc<Player>,
        on_me: bool,
        clicked: &pumpkin_data::item_stack::ItemStack,
        carried: &pumpkin_data::item_stack::ItemStack,
        secondary: bool,
        slot_modifiable: bool,
    ) -> Option<(
        pumpkin_data::item_stack::ItemStack,
        pumpkin_data::item_stack::ItemStack,
    )> {
        let hook = if on_me {
            ItemHooks::STACKED_ON_ME
        } else {
            ItemHooks::STACKED_ON_OTHER
        };
        if !self.hooks.contains(hook) {
            return None;
        }
        match self.invoke_reply(
            server,
            ItemCallData::Stacked {
                player,
                on_me,
                clicked: clicked.clone(),
                carried: carried.clone(),
                secondary,
                slot_modifiable,
            },
        ) {
            ItemReply::Stacked(slot, carried) => Some((slot, carried)),
            _ => None,
        }
    }

    fn destroyed(
        &self,
        server: &Server,
        world: Arc<World>,
        entity: Arc<dyn EntityBase>,
        stack: pumpkin_data::item_stack::ItemStack,
    ) {
        if !self.hooks.contains(ItemHooks::DESTROYED) {
            return;
        }
        self.invoke_reply(
            server,
            ItemCallData::Destroyed {
                world,
                entity,
                stack,
            },
        );
    }
}

impl pumpkin_core::item::ItemBehaviour for PluginItem {
    fn normal_use_with_hand(
        &self,
        _item: &pumpkin_data::item::Item,
        player: &Player,
        _yaw: f32,
        _pitch: f32,
        hand: Hand,
    ) {
        if !self.hooks.contains(ItemHooks::USE) {
            return;
        }
        let world = player.world();
        let Some(server) = world.server.upgrade() else {
            return;
        };
        let Some(player) = world.get_player_by_id(player.entity_id()) else {
            return;
        };
        let stack = player.inventory().get_stack_in_hand(hand);
        self.invoke(
            &server,
            ItemCallData::Use {
                world,
                player,
                hand,
                stack,
            },
        );
    }

    fn use_on_block(
        &self,
        item: &mut pumpkin_data::item_stack::ItemStack,
        player: &Player,
        location: BlockPos,
        face: BlockDirection,
        cursor_pos: Vector3<f32>,
        _block: &Block,
        server: &Server,
    ) -> BlockActionResult {
        if !self.hooks.contains(ItemHooks::USE_ON_BLOCK) {
            return BlockActionResult::Pass;
        }
        let world = player.world();
        let Some(player) = world.get_player_by_id(player.entity_id()) else {
            return BlockActionResult::Pass;
        };
        // The main hand is the one in use unless it holds a different item.
        let hand = if player.inventory().held_item().item.id == item.item.id {
            Hand::Right
        } else {
            Hand::Left
        };
        let state = world.get_block_state_id(&location);
        let reply = self.invoke(
            server,
            ItemCallData::UseOnBlock {
                world,
                pos: location,
                state,
                player,
                hand,
                face,
                location: hit_location(&location, &cursor_pos),
                stack: item.clone(),
            },
        );
        reply.map_or(BlockActionResult::Pass, PluginBlock::interaction)
    }

    fn on_use_tick(
        &self,
        stack: &pumpkin_data::item_stack::ItemStack,
        player: &Player,
        remaining_use_ticks: i32,
    ) {
        if !self.hooks.contains(ItemHooks::USE_TICK) {
            return;
        }
        let Some(hand) = *player
            .living_entity
            .active_hand
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
        else {
            return;
        };
        let world = player.world();
        let Some(player) = world.get_player_by_id(player.entity_id()) else {
            return;
        };
        let item_use = QueuedItemUse {
            handler_id: self.handler_id,
            player,
            hand,
            stack: stack.clone(),
            remaining_ticks: remaining_use_ticks,
        };
        let target: Arc<dyn PluginTickTarget> = self.plugin.clone();
        world
            .plugin_ticks
            .push(&target, |batch| batch.item_uses.push(item_use));
    }

    fn finish_using(
        &self,
        stack: &pumpkin_data::item_stack::ItemStack,
        player: &Player,
        hand: Hand,
    ) -> Option<pumpkin_data::item_stack::ItemStack> {
        // The hook sees the hand after the `consumable` component was applied.
        let in_hand = player.inventory.get_stack_in_hand(hand);
        let in_hand = if in_hand.item.id == stack.item.id {
            in_hand
        } else {
            stack.clone()
        };
        match self.using(ItemHooks::FINISH_USING, player, hand, &in_hand, 0) {
            ItemReply::Stack(stack) => Some(stack),
            _ => None,
        }
    }

    fn use_on_release(&self) -> bool {
        self.hooks.contains(ItemHooks::USE_ON_RELEASE)
    }

    fn on_stopped_using(&self, stack: &pumpkin_data::item_stack::ItemStack, player: &Player) {
        let hand = player
            .living_entity
            .active_hand
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .unwrap_or(Hand::Right);
        let remaining = player
            .living_entity
            .item_use_time
            .load(std::sync::atomic::Ordering::Relaxed);
        self.using(ItemHooks::RELEASE_USING, player, hand, stack, remaining);
    }

    fn on_use_ended(
        &self,
        stack: &pumpkin_data::item_stack::ItemStack,
        player: &Player,
        hand: Hand,
        remaining: i32,
    ) {
        self.using(ItemHooks::STOP_USING, player, hand, stack, remaining);
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn plugin_hooks(&self) -> Option<&dyn PluginItemHooks> {
        Some(self)
    }
}

/// Opens a modded menu the way the player's loader expects: vanilla's container screen plus
/// the loader's own open packet, carrying the menu type and the plugin's extra data.
pub fn open_modded_menu(open: &menu::OpenMenu, menu_type: &str, menu_raw_id: u16, data: &[u8]) {
    use pumpkin_protocol::codec::var_int::VarInt;
    use pumpkin_protocol::java::client::play::CCustomPayload;
    use pumpkin_protocol::ser::NetworkWriteExt;

    let player = open.player.clone();
    player.open_custom_screen(
        |sync_id| open.handler(sync_id),
        |sync_id| {
            let version = pumpkin_util::version::JavaMinecraftVersion::V_26_3;
            let mut payload = Vec::new();
            let (channel, written) = match client_loader(&player) {
                Some(ModLoader::Forge) => {
                    // Forge's `OpenContainer` (`IForgeServerPlayer.openMenu`): the
                    // message id, the menu type's raw id, the window id, the title,
                    // then the extra data as a byte array.
                    let written = payload
                        .write_var_int(&VarInt(pumpkin_forge::wire::OPEN_CONTAINER))
                        .and_then(|()| payload.write_var_int(&VarInt(menu_raw_id.into())))
                        .and_then(|()| payload.write_var_int(&VarInt(sync_id.into())))
                        .and_then(|()| payload.write_component(&open.title, &version))
                        .and_then(|()| payload.write_var_int(&VarInt(data.len() as i32)));
                    (pumpkin_forge::wire::HANDSHAKE_CHANNEL, written)
                }
                Some(ModLoader::NeoForge) => {
                    // NeoForge's `AdvancedOpenScreenPayload` (`ServerPlayer.openMenu`
                    // patch): the window id, the menu type's raw id, the title, then
                    // the extra data as a byte array.
                    let written = payload
                        .write_var_int(&VarInt(sync_id.into()))
                        .and_then(|()| payload.write_var_int(&VarInt(menu_raw_id.into())))
                        .and_then(|()| payload.write_component(&open.title, &version))
                        .and_then(|()| payload.write_var_int(&VarInt(data.len() as i32)));
                    (
                        pumpkin_neoforge::wire::ADVANCED_OPEN_SCREEN_CHANNEL,
                        written,
                    )
                }
                _ => {
                    // fabric-screen-handler-api-v1 `Networking.OpenScreenPayload.write`
                    // (1.21.1).
                    let written = payload
                        .write_string(menu_type)
                        .and_then(|()| payload.write_u8(sync_id))
                        .and_then(|()| payload.write_component(&open.title, &version));
                    ("fabric-screen-handler-api-v1:open_screen", written)
                }
            };
            if let Err(error) = written {
                tracing::error!(%error, "Failed to write a modded menu open packet");
                return;
            }
            payload.extend_from_slice(data);
            player.try_send_client_packet(&CCustomPayload::new(channel, &payload));
        },
    );
}
