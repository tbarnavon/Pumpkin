//! Host side of the `modded` interface: plugin-backed block behaviour and block-entity storage.

use std::sync::Arc;

use pumpkin_data::{Block, BlockDirection, BlockStateId, HorizontalFacingExt};
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_nbt::tag::NbtTag;
use pumpkin_protocol::codec::var_int::VarInt;
use pumpkin_protocol::java::client::play::CBlockEntityData;
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector3::Vector3;
use tokio::sync::Mutex;
use wasmtime::component::Resource;

use crate::block::registry::BlockActionResult;
use crate::block::{
    BlockBehaviour, NormalUseArgs, OnPlaceArgs, OnScheduledTickArgs, PlayerPlacedArgs,
    UseWithItemArgs,
};
use crate::entity::EntityBase;
use crate::entity::player::Player;
use crate::plugin::loader::wasm::wasm_host::{PluginInstance, WasmPlugin, state::PluginHostState};
use crate::server::Server;
use crate::world::World;

use super::common::{from_wit_nbt_tree, to_wit_nbt_tree};
use super::mob::to_wit_block_direction;
use super::pumpkin::plugin::common::{BlockPos as WitBlockPos, Hand as WitHand, NbtTree};
use super::pumpkin::plugin::item_stack::ItemStack as WitItemStack;
use super::pumpkin::plugin::modded::{
    self as wit, BlockCall, BlockHit, BlockHooks, BlockReply, Breaking, Interaction,
    InteractionResult, Placement, Removal, Tick,
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

const fn from_wit_pos(pos: WitBlockPos) -> BlockPos {
    BlockPos::new(pos.x, pos.y, pos.z)
}

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
    },
}

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
    /// Calls the plugin synchronously, the way custom AI goals do (see `mob.rs`).
    fn invoke(&self, server: &Server, data: CallData) -> HookReply {
        let plugin = self.plugin.clone();
        let handler_id = self.handler_id;
        let run = async move {
            let PluginInstance::V0_1(instance) = plugin.plugin_instance.as_ref();
            let function = instance.func_handle_block_hook();
            plugin
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
                            BlockReply::None => HookReply::None,
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
                tracing::error!(handler_id, %error, "Wasm block hook failed");
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
        } => BlockCall::ScheduledTick(Tick {
            world: state.add::<WitWorld>(world)?,
            pos: to_wit_pos(pos),
            state: block_state.as_u16(),
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
            },
        );
    }
}

/// Block-entity data of a modded block at `pos`, without the `id`/`x`/`y`/`z` keys.
fn block_entity_data(world: &World, pos: &BlockPos) -> Option<NbtCompound> {
    world
        .level
        .read_chunk_sync(&pos.chunk_position(), |chunk| {
            chunk
                .pending_block_entities
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .get(pos)
                .cloned()
        })
        .flatten()
        .map(|mut nbt| {
            for key in ["id", "x", "y", "z"] {
                nbt.child_tags.remove(key);
            }
            nbt
        })
}

impl wit::Host for PluginHostState {
    async fn get_block_entity_data(
        &mut self,
        world: Resource<WitWorld>,
        pos: WitBlockPos,
    ) -> wasmtime::Result<Option<NbtTree>> {
        let world = self.get(&world)?.clone();
        Ok(block_entity_data(&world, &from_wit_pos(pos))
            .map(|nbt| to_wit_nbt_tree(NbtTag::Compound(nbt))))
    }

    async fn set_block_entity_data(
        &mut self,
        world: Resource<WitWorld>,
        pos: WitBlockPos,
        block_entity_type: String,
        data: NbtTree,
    ) -> wasmtime::Result<()> {
        let world = self.get(&world)?.clone();
        let pos = from_wit_pos(pos);
        let NbtTag::Compound(mut nbt) = from_wit_nbt_tree(&data).map_err(wasmtime::Error::msg)?
        else {
            return Err(wasmtime::Error::msg("block-entity data must be a compound"));
        };
        let Some(type_id) = pumpkin_data::dynamic::names::block_entity_type_id(&block_entity_type)
        else {
            return Err(wasmtime::Error::msg(format!(
                "unknown block-entity type {block_entity_type}"
            )));
        };
        for key in ["id", "x", "y", "z"] {
            nbt.child_tags.remove(key);
        }
        // Same payload vanilla sends: the block entity's update tag, without id and position.
        let bytes = pumpkin_nbt::Nbt::from(nbt.clone()).write_unnamed();
        world.broadcast_to_chunk(
            pos.chunk_position(),
            &CBlockEntityData::new(pos, VarInt(i32::from(type_id)), bytes.as_ref().into()),
        );
        nbt.put_string("id", block_entity_type);
        nbt.put_int("x", pos.0.x);
        nbt.put_int("y", pos.0.y);
        nbt.put_int("z", pos.0.z);
        world.add_block_entity_nbt(pos, &nbt);
        Ok(())
    }

    async fn remove_block_entity_data(
        &mut self,
        world: Resource<WitWorld>,
        pos: WitBlockPos,
    ) -> wasmtime::Result<()> {
        let world = self.get(&world)?.clone();
        world.remove_pending_block_entity_nbt(&from_wit_pos(pos));
        Ok(())
    }

    async fn get_item_component(
        &mut self,
        stack: Resource<WitItemStack>,
        component: String,
    ) -> wasmtime::Result<Option<NbtTree>> {
        let Some(id) = pumpkin_data::data_component::DataComponent::try_from_name(&component)
        else {
            return Ok(None);
        };
        let stack = self.get(&stack)?.clone();
        let stack = stack.lock().await;
        let value = match stack.patch.iter().find(|(pid, _)| *pid == id) {
            Some((_, value)) => value.as_ref().map(|v| v.write_data()),
            None => stack
                .item
                .components
                .iter()
                .find(|(pid, _)| *pid == id)
                .map(|(_, v)| v.write_data()),
        };
        Ok(value.map(to_wit_nbt_tree))
    }

    async fn set_item_component(
        &mut self,
        stack: Resource<WitItemStack>,
        component: String,
        value: NbtTree,
    ) -> wasmtime::Result<Result<(), String>> {
        let Some(id) = pumpkin_data::data_component::DataComponent::try_from_name(&component)
        else {
            return Ok(Err(format!("unknown item component {component}")));
        };
        let tag = match from_wit_nbt_tree(&value) {
            Ok(tag) => tag,
            Err(error) => return Ok(Err(error)),
        };
        let Some(data) = pumpkin_data::data_component_impl::read_data(id, &tag) else {
            return Ok(Err(format!(
                "cannot read a value for item component {component}"
            )));
        };
        let stack = self.get(&stack)?.clone();
        let mut stack = stack.lock().await;
        match stack.patch.iter_mut().find(|(pid, _)| *pid == id) {
            Some((_, slot)) => *slot = Some(data),
            None => stack.patch.push((id, Some(data))),
        }
        Ok(Ok(()))
    }

    async fn remove_item_component(
        &mut self,
        stack: Resource<WitItemStack>,
        component: String,
    ) -> wasmtime::Result<()> {
        if let Some(id) = pumpkin_data::data_component::DataComponent::try_from_name(&component) {
            let stack = self.get(&stack)?.clone();
            stack.lock().await.patch.retain(|(pid, _)| *pid != id);
        }
        Ok(())
    }

    async fn schedule_block_tick(
        &mut self,
        world: Resource<WitWorld>,
        pos: WitBlockPos,
        delay: u32,
    ) -> wasmtime::Result<()> {
        let world = self.get(&world)?.clone();
        let pos = from_wit_pos(pos);
        let block = world.get_block(&pos);
        world.schedule_block_tick(
            block,
            pos,
            u8::try_from(delay).unwrap_or(u8::MAX),
            pumpkin_world::tick::TickPriority::Normal,
        );
        Ok(())
    }
}
