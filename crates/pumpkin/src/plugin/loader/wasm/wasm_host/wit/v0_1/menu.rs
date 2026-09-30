//! Host side of the `menu` interface: menus whose slots a plugin defines.

use std::any::Any;
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, LazyLock, Mutex as StdMutex, Weak};

use pumpkin_data::item_stack::ItemStack;
use pumpkin_data::screen::WindowType;
use pumpkin_inventory::Clearable;
use pumpkin_inventory::inventory::Inventory;
use pumpkin_inventory::player::player_inventory::PlayerInventory;
use pumpkin_inventory::screen_handler::{
    InventoryPlayer, ScreenHandler, ScreenHandlerBehaviour, ScreenHandlerFactory,
    SharedScreenHandler,
};
use pumpkin_inventory::slot::{NormalSlot, Slot};
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::text::TextComponent;
use tokio::sync::Mutex;
use wasmtime::component::{Access, HasSelf, Resource};

use crate::entity::player::Player;
use crate::plugin::loader::wasm::wasm_host::{PluginInstance, WasmPlugin, state::PluginHostState};
use crate::server::Server;
use crate::world::World;

use super::gui::from_wit_screen;
use super::pumpkin::plugin::item_stack::ItemStack as WitItemStack;
use super::pumpkin::plugin::menu::{
    self as wit, MenuAnchor, MenuCall, MenuDefinition, MenuReply, MenuSlot, QuickMoveStep, SetSlot,
    SlotItem,
};
use super::pumpkin::plugin::player::Player as WitPlayer;
use super::pumpkin::plugin::screens::Screen as WitScreen;

/// Vanilla `AbstractContainerMenu.SLOT_CLICKED_OUTSIDE` aside, slot indexes fit in a byte.
const MAX_SLOT_COUNT: u32 = 256;

/// What the host asks the plugin, with the items as host stacks.
enum Call {
    SetItem(u32, Option<ItemStack>),
    MayPlace(u32, ItemStack),
    MayPickup(u32),
    MaxCount(u32, ItemStack),
    QuickMove(u32, ItemStack),
    Closed,
}

enum Reply {
    None,
    Allowed(bool),
    Count(u32),
    QuickMove(Vec<QuickMoveStep>),
}

/// The plugin slots of every open menu with the same plugin, handler and menu id. The plugin
/// pushes changes with `menu.update-slot`; the host reads them when it syncs the viewers.
struct SharedContents {
    items: StdMutex<Vec<ItemStack>>,
    closed: AtomicBool,
}

impl SharedContents {
    fn set(&self, slot: usize, item: ItemStack) {
        let mut items = self
            .items
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if let Some(entry) = items.get_mut(slot) {
            *entry = item;
        }
    }
}

type ContentsKey = (usize, u32, u32);

/// Open menus' contents by `(plugin, handler id, menu id)`. Entries die with their last menu.
static CONTENTS: LazyLock<StdMutex<HashMap<ContentsKey, Weak<SharedContents>>>> =
    LazyLock::new(|| StdMutex::new(HashMap::new()));

fn contents_key(plugin: &Arc<WasmPlugin>, handler_id: u32, menu_id: u32) -> ContentsKey {
    (Arc::as_ptr(plugin) as usize, handler_id, menu_id)
}

fn find_contents(key: ContentsKey) -> Option<Arc<SharedContents>> {
    CONTENTS
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .get(&key)
        .and_then(Weak::upgrade)
}

/// The shared contents for a menu being opened, set to the items it was opened with.
fn open_contents(key: ContentsKey, items: Vec<ItemStack>) -> Arc<SharedContents> {
    let mut map = CONTENTS
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    map.retain(|_, contents| contents.strong_count() > 0);
    if let Some(contents) = map.get(&key).and_then(Weak::upgrade) {
        let mut current = contents
            .items
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        // Another viewer may have a larger layout; keep its slots.
        for (slot, item) in items.into_iter().enumerate() {
            match current.get_mut(slot) {
                Some(entry) => *entry = item,
                None => current.push(item),
            }
        }
        drop(current);
        contents.closed.store(false, Ordering::Relaxed);
        return contents;
    }
    let contents = Arc::new(SharedContents {
        items: StdMutex::new(items),
        closed: AtomicBool::new(false),
    });
    map.insert(key, Arc::downgrade(&contents));
    contents
}

/// A `menu-anchor` resolved when the menu opens.
struct Anchor {
    world: Weak<World>,
    pos: BlockPos,
    block: pumpkin_data::BlockId,
    max_distance_sq: f64,
}

/// One open plugin menu: who it belongs to and the plugin slots it shows.
pub struct PluginMenu {
    plugin: Arc<WasmPlugin>,
    handler_id: u32,
    menu_id: u32,
    viewer: Arc<Player>,
    server: Arc<Server>,
    size: usize,
    contents: Arc<SharedContents>,
    anchor: Option<Anchor>,
}

impl PluginMenu {
    /// Calls the plugin synchronously, the way block hooks do (see `modded.rs`).
    fn call(&self, call: Call) -> Option<Reply> {
        let plugin = self.plugin.clone();
        let viewer = self.viewer.clone();
        let (handler_id, menu_id) = (self.handler_id, self.menu_id);
        let run = async move {
            let generation = plugin.current();
            let PluginInstance::V0_1(instance) = &generation.plugin_instance;
            let function = instance.func_handle_menu_call();
            generation
                .store
                .call_guest(move |mut guest| {
                    Box::pin(async move {
                        let (player, call) = guest.with(|mut store| {
                            let state = store.data_mut();
                            let player = state.add::<WitPlayer>(viewer)?;
                            let mut stack = |stack: ItemStack| {
                                state.add::<WitItemStack>(Arc::new(Mutex::new(stack)))
                            };
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
                            MenuReply::QuickMove(steps) => Reply::QuickMove(steps),
                        })
                    })
                })
                .await
        };

        let result = if tokio::runtime::Handle::try_current().is_ok() {
            tokio::task::block_in_place(|| self.server.runtime.block_on(run))
        } else {
            self.server.runtime.block_on(run)
        };
        match result {
            Ok(reply) => Some(reply),
            Err(error) => {
                tracing::error!(handler_id, error = ?error, "Wasm menu call failed");
                None
            }
        }
    }

    fn get_item(&self, slot: usize) -> ItemStack {
        self.contents
            .items
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .get(slot)
            .cloned()
            .unwrap_or_else(|| ItemStack::EMPTY.clone())
    }

    /// `AbstractContainerMenu.stillValid`, from the anchor alone.
    fn still_valid(&self) -> bool {
        if self.contents.closed.load(Ordering::Relaxed) {
            return false;
        }
        let Some(anchor) = &self.anchor else {
            return true;
        };
        let Some(world) = anchor.world.upgrade() else {
            return false;
        };
        if !Arc::ptr_eq(&world, &self.viewer.world()) {
            return false;
        }
        if pumpkin_data::BlockId::from_state_id(world.get_block_state_id(&anchor.pos))
            != anchor.block
        {
            return false;
        }
        let center = anchor.pos.to_centered_f64();
        let pos = self.viewer.living_entity.entity.pos.load();
        let (dx, dy, dz) = (pos.x - center.x, pos.y - center.y, pos.z - center.z);
        dx * dx + dy * dy + dz * dz <= anchor.max_distance_sq
    }

    pub(super) fn clone_plugin(&self) -> Arc<WasmPlugin> {
        self.plugin.clone()
    }

    fn allowed(&self, call: Call) -> bool {
        matches!(self.call(call), Some(Reply::Allowed(true)))
    }
}

/// The plugin's slots, seen as one inventory.
struct PluginMenuInventory(Arc<PluginMenu>);

impl Clearable for PluginMenuInventory {
    fn clear(&self) {}
}

impl Inventory for PluginMenuInventory {
    fn size(&self) -> usize {
        self.0.size
    }

    fn is_empty(&self) -> bool {
        self.0
            .contents
            .items
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .iter()
            .all(ItemStack::is_empty)
    }

    fn get_stack(&self, slot: usize) -> ItemStack {
        self.0.get_item(slot)
    }

    fn remove_stack(&self, slot: usize) -> ItemStack {
        let stack = self.get_stack(slot);
        self.set_stack(slot, ItemStack::EMPTY.clone());
        stack
    }

    fn remove_stack_specific(&self, slot: usize, amount: u8) -> ItemStack {
        let mut stack = self.get_stack(slot);
        if stack.is_empty() || amount == 0 {
            return ItemStack::EMPTY.clone();
        }
        let taken = stack.split(amount);
        self.set_stack(slot, stack);
        taken
    }

    fn set_stack(&self, slot: usize, stack: ItemStack) {
        self.0.contents.set(slot, stack.clone());
        let item = (!stack.is_empty()).then_some(stack);
        self.0.call(Call::SetItem(slot as u32, item));
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

/// A slot the plugin owns: `Slot`'s checks are asked of the plugin.
struct PluginMenuSlot {
    menu: Arc<PluginMenu>,
    inventory: Arc<PluginMenuInventory>,
    index: usize,
    id: std::sync::atomic::AtomicUsize,
}

impl Slot for PluginMenuSlot {
    fn get_inventory(&self) -> Arc<dyn Inventory> {
        self.inventory.clone()
    }

    fn get_index(&self) -> usize {
        self.index
    }

    fn set_id(&self, id: usize) {
        self.id.store(id, std::sync::atomic::Ordering::Relaxed);
    }

    fn mark_dirty(&self) {}

    fn can_insert(&self, stack: &ItemStack) -> bool {
        self.menu
            .allowed(Call::MayPlace(self.index as u32, stack.clone()))
    }

    fn can_take_items(&self, _player: &dyn InventoryPlayer) -> bool {
        self.menu.allowed(Call::MayPickup(self.index as u32))
    }

    fn get_max_item_count_for_stack(&self, stack: &ItemStack) -> u8 {
        match self
            .menu
            .call(Call::MaxCount(self.index as u32, stack.clone()))
        {
            Some(Reply::Count(count)) => u8::try_from(count).unwrap_or(u8::MAX),
            _ => 0,
        }
    }
}

pub struct PluginMenuHandler {
    menu: Arc<PluginMenu>,
    behaviour: ScreenHandlerBehaviour,
}

impl PluginMenuHandler {
    fn new(
        sync_id: u8,
        window_type: Option<WindowType>,
        menu: Arc<PluginMenu>,
        slots: &[MenuSlot],
        player_inventory: &Arc<PlayerInventory>,
    ) -> Self {
        let mut behaviour = ScreenHandlerBehaviour::new(sync_id, window_type);
        behaviour.container_slots = menu.size;
        let inventory = Arc::new(PluginMenuInventory(menu.clone()));
        let player_inventory: Arc<dyn Inventory> = player_inventory.clone();
        let slots: Vec<Arc<dyn Slot>> = slots
            .iter()
            .map(|slot| -> Arc<dyn Slot> {
                match *slot {
                    MenuSlot::Player(index) => {
                        Arc::new(NormalSlot::new(player_inventory.clone(), index as usize))
                    }
                    MenuSlot::Plugin(index) => Arc::new(PluginMenuSlot {
                        menu: menu.clone(),
                        inventory: inventory.clone(),
                        index: index as usize,
                        id: std::sync::atomic::AtomicUsize::new(0),
                    }),
                }
            })
            .collect();
        let mut handler = Self { menu, behaviour };
        for slot in slots {
            handler.add_slot(slot);
        }
        handler
    }

    /// Whether the menu is still usable; checked every tick, without calling the plugin.
    #[must_use]
    pub fn still_valid(&self) -> bool {
        self.menu.still_valid()
    }
}

impl ScreenHandler for PluginMenuHandler {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn get_behaviour(&self) -> &ScreenHandlerBehaviour {
        &self.behaviour
    }

    fn get_behaviour_mut(&mut self) -> &mut ScreenHandlerBehaviour {
        &mut self.behaviour
    }

    fn can_use(&self, _player: &dyn InventoryPlayer) -> bool {
        self.still_valid()
    }

    fn on_closed(&mut self, player: &dyn InventoryPlayer) {
        self.default_on_closed(player);
        self.menu.call(Call::Closed);
    }

    /// `AbstractContainerMenu.quickMoveStack`, with the targets chosen by the plugin: the steps
    /// are tried in order until one moves something.
    fn quick_move(&mut self, player: &dyn InventoryPlayer, slot_index: i32) -> ItemStack {
        let Some(slot) = usize::try_from(slot_index)
            .ok()
            .and_then(|i| self.behaviour.slots.get(i).cloned())
        else {
            return ItemStack::EMPTY.clone();
        };
        let mut stack = slot.get_stack();
        if stack.is_empty() {
            return ItemStack::EMPTY.clone();
        }
        let original = stack.clone();
        let Some(Reply::QuickMove(steps)) = self
            .menu
            .call(Call::QuickMove(slot_index as u32, stack.clone()))
        else {
            return ItemStack::EMPTY.clone();
        };

        let slot_count = self.behaviour.slots.len() as u32;
        let mut moved = false;
        for step in steps {
            let (Ok(start), Ok(end)) = (
                i32::try_from(step.start.min(slot_count)),
                i32::try_from(step.end.min(slot_count)),
            ) else {
                continue;
            };
            if step.single {
                let mut one = stack.copy_with_count(1);
                if self.insert_item(&mut one, start, end, step.reverse) {
                    stack.decrement(1);
                    slot.set_stack(stack.clone());
                    slot.on_take_item(player, &stack);
                    return ItemStack::EMPTY.clone();
                }
            } else if self.insert_item(&mut stack, start, end, step.reverse) {
                moved = true;
                break;
            }
        }
        if !moved {
            return ItemStack::EMPTY.clone();
        }

        slot.set_stack(stack.clone());
        if stack.item_count == original.item_count {
            return ItemStack::EMPTY.clone();
        }
        slot.on_take_item(player, &stack);
        original
    }
}

struct Factory {
    menu: Arc<PluginMenu>,
    window_type: WindowType,
    slots: Vec<MenuSlot>,
    title: TextComponent,
}

impl ScreenHandlerFactory for Factory {
    fn create_screen_handler(
        &self,
        sync_id: u8,
        player_inventory: &Arc<PlayerInventory>,
        _player: &dyn InventoryPlayer,
    ) -> Option<SharedScreenHandler> {
        Some(Arc::new(StdMutex::new(PluginMenuHandler::new(
            sync_id,
            Some(self.window_type),
            self.menu.clone(),
            &self.slots,
            player_inventory,
        ))))
    }

    fn get_display_name(&self) -> TextComponent {
        self.title.clone()
    }
}

/// A menu as the plugin defined it, checked and resolved against the host state.
pub(super) struct OpenMenu {
    pub menu: Arc<PluginMenu>,
    pub slots: Vec<MenuSlot>,
    pub title: TextComponent,
    pub player: Arc<Player>,
}

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
    if definition.slots.len() as u32 > MAX_SLOT_COUNT {
        return Ok(Err(format!(
            "a menu has at most {MAX_SLOT_COUNT} slots, got {}",
            definition.slots.len()
        )));
    }
    let mut size = 0;
    for slot in &definition.slots {
        match *slot {
            MenuSlot::Player(index) if index >= 36 => {
                return Ok(Err(format!("player slot {index} is not in 0..36")));
            }
            MenuSlot::Plugin(index) => size = size.max(index as usize + 1),
            MenuSlot::Player(_) => {}
        }
    }
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
    items.resize_with(size, || ItemStack::EMPTY.clone());
    let anchor = definition.anchor.map(|MenuAnchor { pos, max_distance }| {
        let world = player.world();
        let pos = BlockPos::new(pos.x, pos.y, pos.z);
        Anchor {
            block: pumpkin_data::BlockId::from_state_id(world.get_block_state_id(&pos)),
            world: Arc::downgrade(&world),
            pos,
            max_distance_sq: max_distance * max_distance,
        }
    });
    let contents = open_contents(contents_key(&plugin, handler_id, definition.menu_id), items);
    Ok(Ok(OpenMenu {
        menu: Arc::new(PluginMenu {
            plugin,
            handler_id,
            menu_id: definition.menu_id,
            viewer: player.clone(),
            server,
            size,
            contents,
            anchor,
        }),
        slots: definition.slots,
        title,
        player,
    }))
}

impl OpenMenu {
    /// Builds the screen handler for a sync id, for menus opened without `open_handled_screen`.
    pub(super) fn handler(&self, sync_id: u8) -> SharedScreenHandler {
        Arc::new(StdMutex::new(PluginMenuHandler::new(
            sync_id,
            None,
            self.menu.clone(),
            &self.slots,
            &self.player.inventory,
        )))
    }
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
        let Some(contents) = find_contents(contents_key(&plugin, handler_id, menu_id)) else {
            return Ok(());
        };
        let item = match item {
            Some(item) => item.lock().await.clone(),
            None => ItemStack::EMPTY.clone(),
        };
        contents.set(slot as usize, item);
        Ok(())
    }

    async fn close(&mut self, handler_id: u32, menu_id: u32) -> wasmtime::Result<()> {
        let state = self;
        let Some(plugin) = state.plugin.as_ref().and_then(Weak::upgrade) else {
            return Ok(());
        };
        if let Some(contents) = find_contents(contents_key(&plugin, handler_id, menu_id)) {
            // The viewers' next tick sees the menu as no longer valid and closes it.
            contents.closed.store(true, Ordering::Relaxed);
        }
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
        let plugin = open.menu.plugin.clone();
        let factory = Factory {
            menu: open.menu,
            window_type: from_wit_screen(screen),
            slots: open.slots,
            title: open.title,
        };
        let player = open.player;
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
