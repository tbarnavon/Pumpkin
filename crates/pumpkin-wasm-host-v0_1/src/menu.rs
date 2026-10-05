//! Host side of the `menu` interface. The menus themselves are in
//! `pumpkin_wasm_host_common::modded::menu`, shared by every API version; this resolves this
//! version's definitions and calls.

use std::sync::{Arc, Weak};

use pumpkin_data::item_stack::ItemStack;
use pumpkin_util::math::position::BlockPos;
use tokio::sync::Mutex;
use wasmtime::component::{Access, HasSelf, Resource};

use pumpkin_core::entity::player::Player;
use pumpkin_wasm_host_common::modded::menu::{
    self as shared, Call, MenuSlot, OpenMenu, QuickMoveStep, Reply,
};
use pumpkin_wasm_host_common::{plugin::PluginGeneration, state::PluginHostState};

use super::Plugin;
use super::gui::from_wit_screen;
use super::pumpkin::plugin::item_stack::ItemStack as WitItemStack;
use super::pumpkin::plugin::menu::{
    self as wit, MenuAnchor, MenuCall, MenuDefinition, MenuReply, MenuSlot as WitMenuSlot, SetSlot,
    SlotItem,
};
use super::pumpkin::plugin::player::Player as WitPlayer;
use super::pumpkin::plugin::screens::Screen as WitScreen;

/// Calls the plugin's `handle-menu-call`.
pub(crate) async fn call_menu(
    generation: Arc<PluginGeneration>,
    handler_id: u32,
    menu_id: u32,
    viewer: Arc<Player>,
    call: Call,
) -> wasmtime::Result<Reply> {
    let function = generation.instance::<Plugin>().func_handle_menu_call();
    generation
        .store
        .call_guest(move |mut guest| {
            Box::pin(async move {
                let (player, call) = guest.with(|mut store| {
                    let state = store.data_mut();
                    let player = state.add::<WitPlayer>(viewer)?;
                    let mut stack =
                        |stack: ItemStack| state.add::<WitItemStack>(Arc::new(Mutex::new(stack)));
                    Ok::<_, wasmtime::Error>(match call {
                        Call::SetItem(slot, item) => MenuCall::SetItem(SetSlot {
                            slot,
                            item: item.map(&mut stack).transpose()?,
                        }),
                        Call::MayPlace(slot, item) => MenuCall::MayPlace(SlotItem {
                            slot,
                            item: stack(item)?,
                        }),
                        Call::MayPickup(slot) => MenuCall::MayPickup(slot),
                        Call::MaxCount(slot, item) => MenuCall::MaxCount(SlotItem {
                            slot,
                            item: stack(item)?,
                        }),
                        Call::QuickMove(slot, item) => MenuCall::QuickMove(SlotItem {
                            slot,
                            item: stack(item)?,
                        }),
                        Call::Closed => MenuCall::Closed,
                    })
                    .map(|call| (player, call))
                })?;
                let reply = guest
                    .call(function, (handler_id, menu_id, player, call))
                    .await?
                    .0;
                Ok(match reply {
                    MenuReply::None => Reply::None,
                    MenuReply::Allowed(allowed) => Reply::Allowed(allowed),
                    MenuReply::Count(count) => Reply::Count(count),
                    MenuReply::QuickMove(steps) => Reply::QuickMove(
                        steps
                            .into_iter()
                            .map(|step| QuickMoveStep {
                                start: step.start,
                                end: step.end,
                                reverse: step.reverse,
                                single: step.single,
                            })
                            .collect(),
                    ),
                })
            })
        })
        .await
}

/// Checks a `menu-definition` and takes its resources out of the plugin's store.
pub(super) async fn resolve(
    state: &mut PluginHostState,
    player: &Resource<WitPlayer>,
    handler_id: u32,
    definition: MenuDefinition,
) -> wasmtime::Result<Result<OpenMenu, String>> {
    let Some(plugin) = state.plugin.as_ref().and_then(std::sync::Weak::upgrade) else {
        return Ok(Err("plugin is not available".to_string()));
    };
    let Some(server) = state.server.clone() else {
        return Ok(Err("server is not available".to_string()));
    };
    let slots: Vec<MenuSlot> = definition
        .slots
        .iter()
        .map(|slot| match *slot {
            WitMenuSlot::Player(index) => MenuSlot::Player(index),
            WitMenuSlot::Plugin(index) => MenuSlot::Plugin(index),
        })
        .collect();
    let size = match shared::menu_size(&slots) {
        Ok(size) => size,
        Err(error) => return Ok(Err(error)),
    };
    let player = state.get(player)?.clone();
    let title = state.take(definition.title)?;
    let mut handles = Vec::with_capacity(size);
    for item in definition.contents.into_iter().take(size) {
        handles.push(item.map(|item| state.take(item)).transpose()?);
    }
    let mut items = Vec::with_capacity(size);
    for handle in handles {
        items.push(match handle {
            Some(handle) => handle.lock().await.clone(),
            None => ItemStack::EMPTY.clone(),
        });
    }
    let anchor = definition
        .anchor
        .map(|MenuAnchor { pos, max_distance }| (BlockPos::new(pos.x, pos.y, pos.z), max_distance));
    Ok(Ok(OpenMenu::new(
        plugin,
        server,
        player,
        handler_id,
        definition.menu_id,
        slots,
        size,
        items,
        title,
        anchor,
    )))
}

impl wit::Host for PluginHostState {
    async fn update_slot(
        &mut self,
        handler_id: u32,
        menu_id: u32,
        slot: u32,
        item: Option<Resource<WitItemStack>>,
    ) -> wasmtime::Result<()> {
        let state = self;
        let item = item.map(|item| state.take(item)).transpose()?;
        let Some(plugin) = state.plugin.as_ref().and_then(Weak::upgrade) else {
            return Ok(());
        };
        let item = match item {
            Some(item) => item.lock().await.clone(),
            None => ItemStack::EMPTY.clone(),
        };
        shared::update_slot(&plugin, handler_id, menu_id, slot, item);
        Ok(())
    }

    fn close(&mut self, handler_id: u32, menu_id: u32) -> wasmtime::Result<()> {
        let state = self;
        let Some(plugin) = state.plugin.as_ref().and_then(Weak::upgrade) else {
            return Ok(());
        };
        shared::close(&plugin, handler_id, menu_id);
        Ok(())
    }
}

impl wit::HostWithStore<PluginHostState> for HasSelf<PluginHostState> {
    async fn open(
        mut host: Access<'_, PluginHostState, Self>,
        player: Resource<WitPlayer>,
        handler_id: u32,
        screen: WitScreen,
        menu: MenuDefinition,
    ) -> wasmtime::Result<Result<(), String>> {
        let open = match resolve(host.get(), &player, handler_id, menu).await? {
            Ok(open) => open,
            Err(error) => return Ok(Err(error)),
        };
        let plugin = open.plugin();
        let (player, factory) = open.into_factory(from_wit_screen(screen));
        plugin
            .current()
            .store
            .pump_blocking(&mut host, move || {
                player.open_handled_screen(&factory, None);
            })
            .await
            .map(Ok)
    }
}
