//! Host side of the `modded` interface: plugin-backed block and item behaviour, modded menus.

use std::sync::Arc;

use pumpkin_data::{Block, BlockDirection, BlockStateId, HorizontalFacingExt};
use pumpkin_protocol::java::client::play::CCustomPayload;
use pumpkin_protocol::ser::NetworkWriteExt;
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector3::Vector3;
use tokio::sync::Mutex;
use wasmtime::component::{Access, HasSelf, Resource};

use crate::block::registry::BlockActionResult;
use crate::block::{
    BlockBehaviour, EmitsRedstonePowerArgs, GetComparatorOutputArgs, GetRedstonePowerArgs,
    GetStateForNeighborUpdateArgs, NormalUseArgs, OnEntityCollisionArgs, OnEntityStepArgs,
    OnNeighborUpdateArgs, OnPlaceArgs, OnScheduledTickArgs, PlayerPlacedArgs, RandomTickArgs,
    UseWithItemArgs,
};
use crate::entity::EntityBase;
use crate::entity::player::Player;
use crate::plugin::loader::wasm::wasm_host::{PluginInstance, WasmPlugin, state::PluginHostState};
use crate::server::Server;
use crate::world::World;

use super::mob::to_wit_block_direction;
use super::pumpkin::plugin::common::{BlockPos as WitBlockPos, Hand as WitHand};
use super::pumpkin::plugin::item_stack::ItemStack as WitItemStack;
use super::pumpkin::plugin::menu::MenuDefinition;
use super::pumpkin::plugin::modded::{
    self as wit, BlockCall, BlockHit, BlockHooks, BlockReply, Breaking, EntityContact, Interaction,
    InteractionResult, ItemCall, ItemDestroyed, ItemHooks, ItemInventoryTick, ItemStackedClick,
    ItemUse, ItemUseOnBlock, NeighborChange, Placement, Removal, ShapeUpdate, Tick,
};
use super::pumpkin::plugin::player::Player as WitPlayer;
use super::pumpkin::plugin::world::World as WitWorld;

const fn to_wit_pos(pos: BlockPos) -> WitBlockPos {
    WitBlockPos {
        x: pos.0.x,
        y: pos.0.y,
        z: pos.0.z,
    }
}

/// What the `modded` API answers while `[modded] enabled` is off in the server config.
pub const DISABLED: &str = "modded content is disabled ([modded] enabled = false)";

/// A block's hooks as implemented by a Wasm plugin.
pub struct PluginBlock {
    pub plugin: Arc<WasmPlugin>,
    pub handler_id: u32,
    pub hooks: BlockHooks,
}

/// Resources a hook call needs, created in the plugin's store right before the call.
enum CallData {
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
        hand: WitHand,
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
    EntityContact {
        world: Arc<World>,
        pos: BlockPos,
        state: BlockStateId,
        entity_id: i32,
        entity_type: String,
        entity: Arc<dyn EntityBase>,
        step: bool,
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
enum TickKind {
    Scheduled,
    Random,
    BlockEntity,
}

/// Whether any plugin block asked for the `ticker` hook, so worlds only look for plugin block
/// entities to tick when one did.
pub static ANY_TICKER: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

/// What a hook call returned, with item-stack resources already resolved.
pub enum HookReply {
    None,
    State(u16),
    Interaction(InteractionResult),
    Drops(Vec<pumpkin_data::item_stack::ItemStack>),
}

const fn hit(pos: BlockPos, face: BlockDirection, location: Vector3<f64>) -> BlockHit {
    BlockHit {
        pos: to_wit_pos(pos),
        face: to_wit_block_direction(face),
        location: (location.x, location.y, location.z),
    }
}

impl PluginBlock {
    /// Whether `other` is the same plugin's handler, as registered again after a restart.
    #[must_use]
    pub fn is_same_handler(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.plugin, &other.plugin) && self.handler_id == other.handler_id
    }

    /// Calls the plugin synchronously, the way custom AI goals do (see `mob.rs`).
    fn invoke(&self, server: &Server, data: CallData) -> HookReply {
        let plugin = self.plugin.clone();
        let handler_id = self.handler_id;
        let run = async move {
            let generation = plugin.current();
            let PluginInstance::V0_1(instance) = &generation.plugin_instance;
            let function = instance.func_handle_block_hook();
            generation
                .store
                .call_guest(move |mut guest| {
                    Box::pin(async move {
                        // Item stacks are behind an async mutex, so snapshot them here.
                        let tool = match &data {
                            CallData::Breaking { tool, .. } => tool.clone(),
                            _ => None,
                        };
                        let (server_resource, call) = guest.with(|mut store| {
                            let state = store.data_mut();
                            let server = state.server.clone().ok_or_else(|| {
                                wasmtime::Error::msg("Wasm plugin server is not available")
                            })?;
                            let server_resource: Resource<super::pumpkin::plugin::server::Server> =
                                state.add(server)?;
                            let call = build_call(state, data, tool)?;
                            Ok::<_, wasmtime::Error>((server_resource, call))
                        })?;
                        let reply = guest
                            .call(function, (handler_id, server_resource, call))
                            .await?
                            .0;
                        let stacks = match reply {
                            BlockReply::Drops(stacks) => {
                                let handles = guest.with(|mut store| {
                                    stacks
                                        .into_iter()
                                        .map(|stack| store.data_mut().take(stack))
                                        .collect::<wasmtime::Result<Vec<_>>>()
                                })?;
                                let mut drops = Vec::with_capacity(handles.len());
                                for handle in handles {
                                    drops.push(handle.lock().await.clone());
                                }
                                return Ok(HookReply::Drops(drops));
                            }
                            BlockReply::None | BlockReply::Stacked(_) => HookReply::None,
                            BlockReply::State(state) => HookReply::State(state),
                            BlockReply::Interaction(result) => HookReply::Interaction(result),
                        };
                        Ok(stacks)
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

    /// Whether the plugin asked for the `ticker` hook.
    #[must_use]
    pub fn ticks_block_entities(&self) -> bool {
        self.hooks.contains(BlockHooks::TICKER)
    }

    /// The block entity ticker, for a block that opted in with `ticker`.
    pub fn block_entity_tick(
        &self,
        server: &Server,
        world: &Arc<World>,
        pos: BlockPos,
        state: BlockStateId,
    ) {
        self.invoke(
            server,
            CallData::Tick {
                world: world.clone(),
                pos,
                state,
                kind: TickKind::BlockEntity,
            },
        );
    }

    /// `Block.attack`. Returns `true` when the plugin handled the click; the caller then keeps a
    /// creative player from breaking the block.
    pub fn attack(
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
                hand: WitHand::Right,
                face,
                location,
                attack: true,
            },
        );
        matches!(reply, HookReply::Interaction(result) if result != InteractionResult::Pass)
    }

    /// `Block.getDrops`, or `None` to use the loot table.
    pub fn drops(
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
    pub fn removed(&self, server: &Server, world: &Arc<World>, pos: BlockPos, state: BlockStateId) {
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
}

#[allow(clippy::too_many_lines)]
fn build_call(
    state: &mut PluginHostState,
    data: CallData,
    tool: Option<pumpkin_data::item_stack::ItemStack>,
) -> wasmtime::Result<BlockCall> {
    Ok(match data {
        CallData::Placement {
            world,
            pos,
            state: block_state,
            player,
            face,
            location,
            horizontal_facing,
            item,
            placed,
        } => {
            let placement = Placement {
                world: state.add::<WitWorld>(world)?,
                pos: to_wit_pos(pos),
                state: block_state.as_u16(),
                player: player.map(|p| state.add::<WitPlayer>(p)).transpose()?,
                hit: hit(pos, face, location),
                horizontal_facing: to_wit_block_direction(horizontal_facing),
                item: item.map(|i| state.add::<WitItemStack>(i)).transpose()?,
            };
            if placed {
                BlockCall::Placed(placement)
            } else {
                BlockCall::PlacementState(placement)
            }
        }
        CallData::Interaction {
            world,
            pos,
            state: block_state,
            player,
            hand,
            face,
            location,
            attack,
        } => {
            let interaction = Interaction {
                world: state.add::<WitWorld>(world)?,
                pos: to_wit_pos(pos),
                state: block_state.as_u16(),
                player: state.add::<WitPlayer>(player)?,
                hand,
                hit: hit(pos, face, location),
            };
            if attack {
                BlockCall::Attack(interaction)
            } else {
                BlockCall::Use(interaction)
            }
        }
        CallData::Breaking {
            world,
            pos,
            state: block_state,
            player,
            ..
        } => BlockCall::Drops(Breaking {
            world: state.add::<WitWorld>(world)?,
            pos: to_wit_pos(pos),
            state: block_state.as_u16(),
            player: player.map(|p| state.add::<WitPlayer>(p)).transpose()?,
            tool: tool
                .map(|t| state.add::<WitItemStack>(Arc::new(Mutex::new(t))))
                .transpose()?,
        }),
        CallData::Removal {
            world,
            pos,
            state: block_state,
        } => BlockCall::Removed(Removal {
            world: state.add::<WitWorld>(world)?,
            pos: to_wit_pos(pos),
            state: block_state.as_u16(),
        }),
        CallData::Tick {
            world,
            pos,
            state: block_state,
            kind,
        } => {
            let tick = Tick {
                world: state.add::<WitWorld>(world)?,
                pos: to_wit_pos(pos),
                state: block_state.as_u16(),
            };
            match kind {
                TickKind::Scheduled => BlockCall::ScheduledTick(tick),
                TickKind::Random => BlockCall::RandomTick(tick),
                TickKind::BlockEntity => BlockCall::BlockEntityTick(tick),
            }
        }
        CallData::EntityContact {
            world,
            pos,
            state: block_state,
            entity_id,
            entity_type,
            entity,
            step,
        } => {
            let contact = EntityContact {
                world: state.add::<WitWorld>(world)?,
                pos: to_wit_pos(pos),
                state: block_state.as_u16(),
                entity_id,
                entity_type,
                entity: state.add::<super::pumpkin::plugin::world::Entity>(entity)?,
            };
            if step {
                BlockCall::StepOn(contact)
            } else {
                BlockCall::EntityInside(contact)
            }
        }
        CallData::ShapeUpdate {
            world,
            pos,
            state: block_state,
            direction,
            neighbor_pos,
            neighbor_state,
        } => BlockCall::UpdateShape(ShapeUpdate {
            world: state.add::<WitWorld>(world)?,
            pos: to_wit_pos(pos),
            state: block_state.as_u16(),
            direction: to_wit_block_direction(direction),
            neighbor_pos: to_wit_pos(neighbor_pos),
            neighbor_state: neighbor_state.as_u16(),
        }),
        CallData::NeighborChanged {
            world,
            pos,
            state: block_state,
            source,
        } => BlockCall::NeighborChanged(NeighborChange {
            world: state.add::<WitWorld>(world)?,
            pos: to_wit_pos(pos),
            state: block_state.as_u16(),
            source_block: source.to_string(),
        }),
    })
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
        let hand = pumpkin_util::Hand::from_packet_id(args.use_item_on.hand.0).ok();
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
        let item = [pumpkin_util::Hand::Right, pumpkin_util::Hand::Left]
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
                hand: WitHand::Right,
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
        args.world.plugin_signals(args.position).weak(args.direction)
    }

    fn get_strong_redstone_power(&self, args: GetRedstonePowerArgs<'_>) -> u8 {
        if !self.hooks.contains(BlockHooks::SIGNAL_SOURCE) {
            return 0;
        }
        args.world.plugin_signals(args.position).strong(args.direction)
    }

    fn get_comparator_output(&self, args: GetComparatorOutputArgs<'_>) -> Option<u8> {
        self.hooks
            .contains(BlockHooks::ANALOG_OUTPUT)
            .then(|| args.world.plugin_signals(args.position).comparator)
    }

    fn on_entity_collision(&self, args: OnEntityCollisionArgs<'_>) {
        if !self.hooks.contains(BlockHooks::ENTITY_INSIDE) {
            return;
        }
        let entity = args.entity.get_entity();
        let Some(handle) = args.world.get_entity_by_id(entity.entity_id) else {
            return;
        };
        self.invoke(
            args.server,
            CallData::EntityContact {
                world: args.world.clone(),
                pos: *args.position,
                state: args.state.id,
                entity_id: entity.entity_id,
                entity_type: format!("minecraft:{}", entity.entity_type.resource_name),
                entity: handle,
                step: false,
            },
        );
    }

    fn on_entity_step(&self, args: OnEntityStepArgs<'_>) {
        if !self.hooks.contains(BlockHooks::STEP_ON) {
            return;
        }
        let Some(server) = args.world.server.upgrade() else {
            return;
        };
        let entity = args.entity.get_entity();
        let Some(handle) = args.world.get_entity_by_id(entity.entity_id) else {
            return;
        };
        self.invoke(
            &server,
            CallData::EntityContact {
                world: args.world.clone(),
                pos: *args.position,
                state: args.state.id,
                entity_id: entity.entity_id,
                entity_type: format!("minecraft:{}", entity.entity_type.resource_name),
                entity: handle,
                step: true,
            },
        );
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

enum ItemCallData {
    UseOnBlock {
        world: Arc<World>,
        pos: BlockPos,
        state: BlockStateId,
        player: Arc<Player>,
        hand: WitHand,
        face: BlockDirection,
        location: Vector3<f64>,
        stack: pumpkin_data::item_stack::ItemStack,
    },
    Use {
        world: Arc<World>,
        player: Arc<Player>,
        hand: WitHand,
        stack: pumpkin_data::item_stack::ItemStack,
    },
    InventoryTick {
        world: Arc<World>,
        player: Arc<Player>,
        slot: u32,
        selected: bool,
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
}

/// What an item hook call returned, with item-stack resources already resolved.
enum ItemReply {
    None,
    Interaction(InteractionResult),
    Stacked(
        pumpkin_data::item_stack::ItemStack,
        pumpkin_data::item_stack::ItemStack,
    ),
}

fn add_optional_stack(
    state: &mut PluginHostState,
    stack: pumpkin_data::item_stack::ItemStack,
) -> wasmtime::Result<Option<Resource<WitItemStack>>> {
    if stack.is_empty() {
        Ok(None)
    } else {
        Ok(Some(
            state.add::<WitItemStack>(Arc::new(Mutex::new(stack)))?,
        ))
    }
}

fn build_item_call(state: &mut PluginHostState, data: ItemCallData) -> wasmtime::Result<ItemCall> {
    Ok(match data {
        ItemCallData::UseOnBlock {
            world,
            pos,
            state: block_state,
            player,
            hand,
            face,
            location,
            stack,
        } => ItemCall::UseOnBlock(ItemUseOnBlock {
            world: state.add::<WitWorld>(world)?,
            pos: to_wit_pos(pos),
            state: block_state.as_u16(),
            player: state.add::<WitPlayer>(player)?,
            hand,
            hit: hit(pos, face, location),
            stack: state.add::<WitItemStack>(Arc::new(Mutex::new(stack)))?,
        }),
        ItemCallData::Use {
            world,
            player,
            hand,
            stack,
        } => ItemCall::Use(ItemUse {
            world: state.add::<WitWorld>(world)?,
            player: state.add::<WitPlayer>(player)?,
            hand,
            stack: state.add::<WitItemStack>(Arc::new(Mutex::new(stack)))?,
        }),
        ItemCallData::InventoryTick {
            world,
            player,
            slot,
            selected,
            stack,
        } => ItemCall::InventoryTick(ItemInventoryTick {
            world: state.add::<WitWorld>(world)?,
            player: state.add::<WitPlayer>(player)?,
            slot,
            selected,
            stack: state.add::<WitItemStack>(Arc::new(Mutex::new(stack)))?,
        }),
        ItemCallData::Stacked {
            player,
            on_me,
            clicked,
            carried,
            secondary,
            slot_modifiable,
        } => {
            let click = ItemStackedClick {
                player: state.add::<WitPlayer>(player)?,
                slot_stack: add_optional_stack(state, clicked)?,
                carried: add_optional_stack(state, carried)?,
                secondary,
                slot_modifiable,
            };
            if on_me {
                ItemCall::StackedOnMe(click)
            } else {
                ItemCall::StackedOnOther(click)
            }
        }
        ItemCallData::Destroyed {
            world,
            entity,
            stack,
        } => ItemCall::Destroyed(ItemDestroyed {
            world: state.add::<WitWorld>(world)?,
            entity: state.add::<super::pumpkin::plugin::world::Entity>(entity)?,
            stack: state.add::<WitItemStack>(Arc::new(Mutex::new(stack)))?,
        }),
    })
}

impl PluginItem {
    /// Whether both behaviours are the same plugin's handler, as registered again after a restart.
    #[must_use]
    pub fn same_handler(
        a: &dyn crate::item::ItemBehaviour,
        b: &dyn crate::item::ItemBehaviour,
    ) -> bool {
        match (
            a.as_any().downcast_ref::<Self>(),
            b.as_any().downcast_ref::<Self>(),
        ) {
            (Some(a), Some(b)) => Arc::ptr_eq(&a.plugin, &b.plugin) && a.handler_id == b.handler_id,
            _ => false,
        }
    }

    /// `Item.inventoryTick` for a stack in `player`'s inventory, when the plugin opted in.
    pub fn inventory_tick(
        &self,
        server: &Server,
        player: &Arc<Player>,
        slot: usize,
        selected: bool,
        stack: pumpkin_data::item_stack::ItemStack,
    ) {
        if !self.hooks.contains(ItemHooks::INVENTORY_TICK) {
            return;
        }
        self.invoke(
            server,
            ItemCallData::InventoryTick {
                world: player.world(),
                player: player.clone(),
                slot: slot as u32,
                selected,
                stack,
            },
        );
    }

    /// Whether the plugin asked for `inventory-tick` calls.
    #[must_use]
    pub fn ticks_in_inventory(&self) -> bool {
        self.hooks.contains(ItemHooks::INVENTORY_TICK)
    }

    /// `Item.canFitInsideContainerItems`.
    #[must_use]
    pub fn fits_in_containers(&self) -> bool {
        !self.hooks.contains(ItemHooks::NOT_IN_CONTAINERS)
    }

    /// `Item.overrideOtherStackedOnMe` (`on_me`, this item is in the slot) or
    /// `Item.overrideStackedOnOther` (this item is carried). The new slot and cursor stacks when
    /// the plugin took the click over.
    #[expect(clippy::too_many_arguments)]
    pub fn stacked_click(
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

    /// `Item.onDestroyed`: an item entity of this item is about to be removed after damage.
    pub fn destroyed(
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

    fn invoke(&self, server: &Server, data: ItemCallData) -> Option<InteractionResult> {
        match self.invoke_reply(server, data) {
            ItemReply::Interaction(result) => Some(result),
            _ => None,
        }
    }

    fn invoke_reply(&self, server: &Server, data: ItemCallData) -> ItemReply {
        let plugin = self.plugin.clone();
        let handler_id = self.handler_id;
        let run = async move {
            let generation = plugin.current();
            let PluginInstance::V0_1(instance) = &generation.plugin_instance;
            let function = instance.func_handle_item_hook();
            generation
                .store
                .call_guest(move |mut guest| {
                    Box::pin(async move {
                        let (server_resource, call) = guest.with(|mut store| {
                            let state = store.data_mut();
                            let server = state.server.clone().ok_or_else(|| {
                                wasmtime::Error::msg("Wasm plugin server is not available")
                            })?;
                            let server_resource: Resource<super::pumpkin::plugin::server::Server> =
                                state.add(server)?;
                            let call = build_item_call(state, data)?;
                            Ok::<_, wasmtime::Error>((server_resource, call))
                        })?;
                        let reply = guest
                            .call(function, (handler_id, server_resource, call))
                            .await?
                            .0;
                        Ok(match reply {
                            BlockReply::Interaction(result) => ItemReply::Interaction(result),
                            BlockReply::Stacked(result) => {
                                let (slot, carried) = guest.with(|mut store| {
                                    let state = store.data_mut();
                                    let slot =
                                        result.slot_stack.map(|s| state.take(s)).transpose()?;
                                    let carried =
                                        result.carried.map(|s| state.take(s)).transpose()?;
                                    Ok::<_, wasmtime::Error>((slot, carried))
                                })?;
                                let empty = pumpkin_data::item_stack::ItemStack::EMPTY;
                                let slot = match slot {
                                    Some(stack) => stack.lock().await.clone(),
                                    None => empty.clone(),
                                };
                                let carried = match carried {
                                    Some(stack) => stack.lock().await.clone(),
                                    None => empty.clone(),
                                };
                                ItemReply::Stacked(slot, carried)
                            }
                            _ => ItemReply::None,
                        })
                    })
                })
                .await
        };
        let result = if tokio::runtime::Handle::try_current().is_ok() {
            tokio::task::block_in_place(|| server.runtime.block_on(run))
        } else {
            server.runtime.block_on(run)
        };
        result.unwrap_or_else(|error| {
            tracing::error!(handler_id, error = ?error, "Wasm item hook failed");
            ItemReply::None
        })
    }
}

const fn to_wit_hand(hand: pumpkin_util::Hand) -> WitHand {
    match hand {
        pumpkin_util::Hand::Left => WitHand::Left,
        pumpkin_util::Hand::Right => WitHand::Right,
    }
}

impl crate::item::ItemBehaviour for PluginItem {
    fn normal_use_with_hand(
        &self,
        _item: &pumpkin_data::item::Item,
        player: &Player,
        _yaw: f32,
        _pitch: f32,
        hand: pumpkin_util::Hand,
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
                hand: to_wit_hand(hand),
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
            pumpkin_util::Hand::Right
        } else {
            pumpkin_util::Hand::Left
        };
        let state = world.get_block_state_id(&location);
        let reply = self.invoke(
            server,
            ItemCallData::UseOnBlock {
                world,
                pos: location,
                state,
                player,
                hand: to_wit_hand(hand),
                face,
                location: hit_location(&location, &cursor_pos),
                stack: item.clone(),
            },
        );
        reply.map_or(BlockActionResult::Pass, PluginBlock::interaction)
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl wit::Host for PluginHostState {
    async fn set_block_collision_shape(
        &mut self,
        world: Resource<WitWorld>,
        pos: WitBlockPos,
        boxes: Option<Vec<wit::ShapeBox>>,
    ) -> wasmtime::Result<()> {
        use pumpkin_util::math::boundingbox::BoundingBox;
        let world = self.get(&world)?.clone();
        let boxes = boxes.map(|boxes| {
            boxes
                .into_iter()
                .map(|b| BoundingBox {
                    min: Vector3::new(b.min.0, b.min.1, b.min.2),
                    max: Vector3::new(b.max.0, b.max.1, b.max.2),
                })
                .collect()
        });
        world.set_plugin_collision_shapes(&BlockPos::new(pos.x, pos.y, pos.z), boxes);
        Ok(())
    }

    async fn register_component_stream_codec(
        &mut self,
        component: String,
        nodes: Vec<wit::StreamCodecNode>,
    ) -> wasmtime::Result<Result<(), String>> {
        use pumpkin_protocol::codec::modded_component::{ComponentStreamCodec, StreamCodecNode};

        if !self
            .server
            .as_ref()
            .is_some_and(|server| server.advanced_config.modded.enabled)
        {
            return Ok(Err(DISABLED.to_string()));
        }
        let Some(raw_id) = pumpkin_data::item_stack::unknown_component_id(&component) else {
            return Ok(Err(format!("unknown modded component type {component}")));
        };
        let nodes = nodes
            .into_iter()
            .map(|node| match node {
                wit::StreamCodecNode::Bool => StreamCodecNode::Bool,
                wit::StreamCodecNode::Byte => StreamCodecNode::Byte,
                wit::StreamCodecNode::Short => StreamCodecNode::Short,
                wit::StreamCodecNode::Int => StreamCodecNode::Int,
                wit::StreamCodecNode::Long => StreamCodecNode::Long,
                wit::StreamCodecNode::Float => StreamCodecNode::Float,
                wit::StreamCodecNode::Double => StreamCodecNode::Double,
                wit::StreamCodecNode::VarInt => StreamCodecNode::VarInt,
                wit::StreamCodecNode::VarLong => StreamCodecNode::VarLong,
                wit::StreamCodecNode::String => StreamCodecNode::String,
                wit::StreamCodecNode::Nbt => StreamCodecNode::Nbt,
                wit::StreamCodecNode::ItemStack => StreamCodecNode::ItemStack,
                wit::StreamCodecNode::OptionalItemStack => StreamCodecNode::OptionalItemStack,
                wit::StreamCodecNode::Uuid => StreamCodecNode::Uuid,
                wit::StreamCodecNode::BlockPos => StreamCodecNode::BlockPos,
                wit::StreamCodecNode::Composite(fields) => StreamCodecNode::Composite(fields),
                wit::StreamCodecNode::List(child) => StreamCodecNode::List(child),
                wit::StreamCodecNode::Optional(child) => StreamCodecNode::Optional(child),
            })
            .collect();
        Ok(ComponentStreamCodec::register(
            raw_id,
            ComponentStreamCodec { nodes },
        ))
    }
}

impl wit::HostWithStore<PluginHostState> for HasSelf<PluginHostState> {
    async fn open_menu(
        mut host: Access<'_, PluginHostState, Self>,
        player: Resource<WitPlayer>,
        handler_id: u32,
        menu_type: String,
        data: Vec<u8>,
        menu: MenuDefinition,
    ) -> wasmtime::Result<Result<(), String>> {
        if !host
            .get()
            .server
            .as_ref()
            .is_some_and(|server| server.advanced_config.modded.enabled)
        {
            return Ok(Err(DISABLED.to_string()));
        }
        if pumpkin_data::dynamic::names::modded_id(
            pumpkin_data::dynamic::names::SyncedRegistry::Menu,
            &menu_type,
        )
        .is_none()
        {
            return Ok(Err(format!("unknown modded menu type {menu_type}")));
        }
        let open = match super::menu::resolve(host.get(), &player, handler_id, menu).await? {
            Ok(open) => open,
            Err(error) => return Ok(Err(error)),
        };
        let plugin = open.menu.clone_plugin();
        plugin
            .current()
            .store
            .pump_blocking(&mut host, move || {
                let player = open.player.clone();
                player.open_custom_screen(
                    |sync_id| open.handler(sync_id),
                    |sync_id| {
                        // fabric-menu-api-v1 `Networking.OpenScreenPayload.write`.
                        let mut payload = Vec::new();
                        let written = payload
                            .write_string(&menu_type)
                            .and_then(|()| payload.write_u8(sync_id))
                            .and_then(|()| {
                                payload.write_component(
                                    &open.title,
                                    &pumpkin_util::version::JavaMinecraftVersion::V_26_3,
                                )
                            });
                        if let Err(error) = written {
                            tracing::error!(%error, "Failed to write a modded menu open packet");
                            return;
                        }
                        payload.extend_from_slice(&data);
                        player.try_send_client_packet(&CCustomPayload::new(
                            "fabric-menu-api-v1:open_screen",
                            &payload,
                        ));
                    },
                );
            })
            .await
            .map(Ok)
    }
}
