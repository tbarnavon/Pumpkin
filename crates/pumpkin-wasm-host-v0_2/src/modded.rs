//! Host side of the `modded` interface. The behaviour itself is in
//! `pumpkin_wasm_host_common::modded`, shared by every API version; this builds this version's
//! calls and reads its replies ([`GUEST`]).

use std::sync::Arc;

use pumpkin_data::BlockDirection;
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector3::Vector3;
use tokio::sync::Mutex;
use wasmtime::component::{Access, HasSelf, Resource};

use pumpkin_core::entity::player::Player;
use pumpkin_core::plugin::modded::QueuedBatch;
use pumpkin_core::world::World;
use pumpkin_wasm_host_common::modded::{
    self as shared, CallData, DISABLED, GuestCalls, GuestFuture, HookReply, ItemCallData,
    ItemReply, TickKind,
};
use pumpkin_wasm_host_common::{plugin::PluginGeneration, state::PluginHostState};

use super::Plugin;
use super::mob::to_wit_block_direction;
use super::pumpkin::plugin::common::{BlockPos as WitBlockPos, Hand as WitHand};
use super::pumpkin::plugin::item_stack::ItemStack as WitItemStack;
use super::pumpkin::plugin::menu::MenuDefinition;
use super::pumpkin::plugin::modded::{
    self as wit, BlockCall, BlockEntityTick, BlockHit, BlockHooks, BlockReply, Breaking,
    EntityContact, Interaction, InteractionResult, InventoryTick, ItemCall, ItemDestroyed,
    ItemHooks, ItemStackedClick, ItemUse, ItemUseOnBlock, ItemUseTick, ItemUsing, NeighborChange,
    Placement, PlayerInventoryTicks, Removal, ShapeUpdate, Tick, TickBatch,
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

const fn to_wit_hand(hand: pumpkin_util::Hand) -> WitHand {
    match hand {
        pumpkin_util::Hand::Left => WitHand::Left,
        pumpkin_util::Hand::Right => WitHand::Right,
    }
}

const fn from_wit_interaction(result: InteractionResult) -> shared::InteractionResult {
    match result {
        InteractionResult::Pass => shared::InteractionResult::Pass,
        InteractionResult::Success => shared::InteractionResult::Success,
        InteractionResult::Consume => shared::InteractionResult::Consume,
        InteractionResult::Fail => shared::InteractionResult::Fail,
    }
}

/// This version's `block-hooks` as the host's.
pub(super) fn from_wit_block_hooks(hooks: BlockHooks) -> shared::BlockHooks {
    [
        (
            BlockHooks::PLACEMENT_STATE,
            shared::BlockHooks::PLACEMENT_STATE,
        ),
        (BlockHooks::PLACED, shared::BlockHooks::PLACED),
        (BlockHooks::USE, shared::BlockHooks::USE),
        (BlockHooks::ATTACK, shared::BlockHooks::ATTACK),
        (BlockHooks::DROPS, shared::BlockHooks::DROPS),
        (BlockHooks::REMOVED, shared::BlockHooks::REMOVED),
        (
            BlockHooks::SCHEDULED_TICK,
            shared::BlockHooks::SCHEDULED_TICK,
        ),
        (
            BlockHooks::NEIGHBOR_CHANGED,
            shared::BlockHooks::NEIGHBOR_CHANGED,
        ),
        (BlockHooks::RANDOM_TICK, shared::BlockHooks::RANDOM_TICK),
        (BlockHooks::UPDATE_SHAPE, shared::BlockHooks::UPDATE_SHAPE),
        (BlockHooks::SIGNAL_SOURCE, shared::BlockHooks::SIGNAL_SOURCE),
        (BlockHooks::ANALOG_OUTPUT, shared::BlockHooks::ANALOG_OUTPUT),
        (BlockHooks::ENTITY_INSIDE, shared::BlockHooks::ENTITY_INSIDE),
        (BlockHooks::STEP_ON, shared::BlockHooks::STEP_ON),
        (BlockHooks::TICKER, shared::BlockHooks::TICKER),
    ]
    .into_iter()
    .filter(|(wit, _)| hooks.contains(*wit))
    .fold(shared::BlockHooks::empty(), |all, (_, hook)| all | hook)
}

/// This version's `item-hooks` as the host's.
pub(super) fn from_wit_item_hooks(hooks: ItemHooks) -> shared::ItemHooks {
    [
        (ItemHooks::USE_ON_BLOCK, shared::ItemHooks::USE_ON_BLOCK),
        (ItemHooks::USE, shared::ItemHooks::USE),
        (ItemHooks::INVENTORY_TICK, shared::ItemHooks::INVENTORY_TICK),
        (ItemHooks::STACKED_ON_ME, shared::ItemHooks::STACKED_ON_ME),
        (
            ItemHooks::STACKED_ON_OTHER,
            shared::ItemHooks::STACKED_ON_OTHER,
        ),
        (ItemHooks::DESTROYED, shared::ItemHooks::DESTROYED),
        (
            ItemHooks::NOT_IN_CONTAINERS,
            shared::ItemHooks::NOT_IN_CONTAINERS,
        ),
        (ItemHooks::FINISH_USING, shared::ItemHooks::FINISH_USING),
        (ItemHooks::RELEASE_USING, shared::ItemHooks::RELEASE_USING),
        (ItemHooks::USE_TICK, shared::ItemHooks::USE_TICK),
        (ItemHooks::STOP_USING, shared::ItemHooks::STOP_USING),
        (ItemHooks::USE_ON_RELEASE, shared::ItemHooks::USE_ON_RELEASE),
    ]
    .into_iter()
    .filter(|(wit, _)| hooks.contains(*wit))
    .fold(shared::ItemHooks::empty(), |all, (_, hook)| all | hook)
}

/// This version's calls for the shared modded runtime.
pub struct Guest;

/// The [`GuestCalls`] of every plugin built against this API version.
pub static GUEST: Guest = Guest;

impl GuestCalls for Guest {
    fn tick_batch(
        &self,
        generation: Arc<PluginGeneration>,
        world: Arc<World>,
        batch: QueuedBatch,
    ) -> GuestFuture<()> {
        Box::pin(call_tick_batch(generation, world, batch))
    }

    fn block_hook(
        &self,
        generation: Arc<PluginGeneration>,
        handler_id: u32,
        data: CallData,
    ) -> GuestFuture<HookReply> {
        Box::pin(call_block_hook(generation, handler_id, data))
    }

    fn item_hook(
        &self,
        generation: Arc<PluginGeneration>,
        handler_id: u32,
        data: ItemCallData,
    ) -> GuestFuture<ItemReply> {
        Box::pin(call_item_hook(generation, handler_id, data))
    }

    fn menu_call(
        &self,
        generation: Arc<PluginGeneration>,
        handler_id: u32,
        menu_id: u32,
        viewer: Arc<Player>,
        call: pumpkin_wasm_host_common::modded::menu::Call,
    ) -> GuestFuture<pumpkin_wasm_host_common::modded::menu::Reply> {
        Box::pin(super::menu::call_menu(
            generation, handler_id, menu_id, viewer, call,
        ))
    }
}

/// Sends a world's queued per-tick hooks to the plugin as one `handle-tick-batch` call.
async fn call_tick_batch(
    generation: Arc<PluginGeneration>,
    world: Arc<World>,
    batch: QueuedBatch,
) -> wasmtime::Result<()> {
    let function = generation.instance::<Plugin>().func_handle_tick_batch();
    generation
        .store
        .call_guest(move |mut guest| {
            Box::pin(async move {
                let (server_resource, batch) = guest.with(|mut store| {
                    let state = store.data_mut();
                    let server = state.server.clone().ok_or_else(|| {
                        wasmtime::Error::msg("Wasm plugin server is not available")
                    })?;
                    let server_resource: Resource<super::pumpkin::plugin::server::Server> =
                        state.add(server)?;
                    let batch = build_tick_batch(state, world, batch)?;
                    Ok::<_, wasmtime::Error>((server_resource, batch))
                })?;
                guest.call(function, (server_resource, batch)).await?;
                Ok(())
            })
        })
        .await
}

fn build_tick_batch(
    state: &mut PluginHostState,
    world: Arc<World>,
    batch: QueuedBatch,
) -> wasmtime::Result<TickBatch> {
    let mut inventories = Vec::with_capacity(batch.inventories.len());
    for (player, ticks) in batch.inventories {
        let mut wit_ticks = Vec::with_capacity(ticks.len());
        for tick in ticks {
            wit_ticks.push(InventoryTick {
                handler_id: tick.handler_id,
                slot: tick.slot,
                selected: tick.selected,
                stack: state.add::<WitItemStack>(Arc::new(Mutex::new(tick.stack)))?,
            });
        }
        inventories.push(PlayerInventoryTicks {
            player: state.add::<WitPlayer>(player)?,
            ticks: wit_ticks,
        });
    }
    let mut item_uses = Vec::with_capacity(batch.item_uses.len());
    for item_use in batch.item_uses {
        item_uses.push(ItemUseTick {
            handler_id: item_use.handler_id,
            player: state.add::<WitPlayer>(item_use.player)?,
            hand: to_wit_hand(item_use.hand),
            stack: state.add::<WitItemStack>(Arc::new(Mutex::new(item_use.stack)))?,
            remaining_ticks: item_use.remaining_ticks,
        });
    }
    let mut entity_contacts = Vec::with_capacity(batch.entity_contacts.len());
    for contact in batch.entity_contacts {
        let entity = contact.entity.get_entity();
        let entity_id = entity.entity_id;
        let entity_type = format!("minecraft:{}", entity.entity_type.resource_name);
        entity_contacts.push(EntityContact {
            handler_id: contact.handler_id,
            step: contact.step,
            pos: to_wit_pos(contact.pos),
            state: contact.state.as_u16(),
            entity_id,
            entity_type,
            entity: state.add::<super::pumpkin::plugin::world::Entity>(contact.entity)?,
        });
    }
    let block_entities = batch
        .block_entities
        .into_iter()
        .map(|(handler_id, pos, block_state)| BlockEntityTick {
            handler_id,
            pos: to_wit_pos(pos),
            state: block_state.as_u16(),
        })
        .collect();
    Ok(TickBatch {
        world: state.add::<WitWorld>(world)?,
        inventories,
        item_uses,
        entity_contacts,
        block_entities,
    })
}

const fn hit(pos: BlockPos, face: BlockDirection, location: Vector3<f64>) -> BlockHit {
    BlockHit {
        pos: to_wit_pos(pos),
        face: to_wit_block_direction(face),
        location: (location.x, location.y, location.z),
    }
}

/// Calls the plugin's `handle-block-hook`.
async fn call_block_hook(
    generation: Arc<PluginGeneration>,
    handler_id: u32,
    data: CallData,
) -> wasmtime::Result<HookReply> {
    let function = generation.instance::<Plugin>().func_handle_block_hook();
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
                    BlockReply::None | BlockReply::Stacked(_) | BlockReply::Stack(_) => {
                        HookReply::None
                    }
                    BlockReply::State(state) => HookReply::State(state),
                    BlockReply::Interaction(result) => {
                        HookReply::Interaction(from_wit_interaction(result))
                    }
                };
                Ok(stacks)
            })
        })
        .await
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
                hand: to_wit_hand(hand),
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
            hand: to_wit_hand(hand),
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
            hand: to_wit_hand(hand),
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
        ItemCallData::Using {
            hook,
            world,
            player,
            hand,
            stack,
            remaining_ticks,
        } => {
            let using = ItemUsing {
                world: state.add::<WitWorld>(world)?,
                player: state.add::<WitPlayer>(player)?,
                hand: to_wit_hand(hand),
                stack: state.add::<WitItemStack>(Arc::new(Mutex::new(stack)))?,
                remaining_ticks,
            };
            if hook == shared::ItemHooks::FINISH_USING {
                ItemCall::FinishUsing(using)
            } else if hook == shared::ItemHooks::RELEASE_USING {
                ItemCall::ReleaseUsing(using)
            } else {
                ItemCall::StopUsing(using)
            }
        }
    })
}

/// Calls the plugin's `handle-item-hook`.
async fn call_item_hook(
    generation: Arc<PluginGeneration>,
    handler_id: u32,
    data: ItemCallData,
) -> wasmtime::Result<ItemReply> {
    let function = generation.instance::<Plugin>().func_handle_item_hook();
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
                    BlockReply::Interaction(result) => {
                        ItemReply::Interaction(from_wit_interaction(result))
                    }
                    BlockReply::Stacked(result) => {
                        let (slot, carried) = guest.with(|mut store| {
                            let state = store.data_mut();
                            let slot = result.slot_stack.map(|s| state.take(s)).transpose()?;
                            let carried = result.carried.map(|s| state.take(s)).transpose()?;
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
                    BlockReply::Stack(stack) => {
                        let stack = guest.with(|mut store| {
                            stack.map(|s| store.data_mut().take(s)).transpose()
                        })?;
                        ItemReply::Stack(match stack {
                            Some(stack) => stack.lock().await.clone(),
                            None => pumpkin_data::item_stack::ItemStack::EMPTY.clone(),
                        })
                    }
                    _ => ItemReply::None,
                })
            })
        })
        .await
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

    async fn register_loader_channels(
        &mut self,
        loader: wit::Loader,
        channels: Vec<wit::LoaderChannel>,
    ) -> wasmtime::Result<Result<(), String>> {
        use pumpkin_core::net::java::loaders::{ModChannel, ModLoader};

        if !self
            .server
            .as_ref()
            .is_some_and(|server| server.advanced_config.modded.enabled)
        {
            return Ok(Err(DISABLED.to_string()));
        }
        let loader = match loader {
            wit::Loader::Fabric => ModLoader::Fabric,
            wit::Loader::Neoforge => ModLoader::NeoForge,
            wit::Loader::Forge => ModLoader::Forge,
        };
        let mut parsed = Vec::with_capacity(channels.len());
        for channel in channels {
            if loader == ModLoader::Forge && channel.version.parse::<i32>().is_err() {
                return Ok(Err(format!(
                    "Forge channel {} needs a numeric version, not {}",
                    channel.id, channel.version
                )));
            }
            parsed.push(ModChannel {
                id: channel.id,
                version: channel.version,
                to_client: !matches!(channel.flow, wit::ChannelFlow::ToServer),
                to_server: !matches!(channel.flow, wit::ChannelFlow::ToClient),
                optional: channel.optional,
            });
        }
        pumpkin_core::net::java::loaders::register_mod_channels(loader, parsed);
        Ok(Ok(()))
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
        let Some(menu_raw_id) = pumpkin_data::dynamic::names::modded_id(
            pumpkin_data::dynamic::names::SyncedRegistry::Menu,
            &menu_type,
        ) else {
            return Ok(Err(format!("unknown modded menu type {menu_type}")));
        };
        let open = match super::menu::resolve(host.get(), &player, handler_id, menu).await? {
            Ok(open) => open,
            Err(error) => return Ok(Err(error)),
        };
        let plugin = open.plugin();
        plugin
            .current()
            .store
            .pump_blocking(&mut host, move || {
                shared::open_modded_menu(&open, &menu_type, menu_raw_id, &data);
            })
            .await
            .map(Ok)
    }
}
