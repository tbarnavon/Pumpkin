//! Menus whose slots a plugin defines, shared by every API version.
//!
//! Like [`super::PluginBlock`], everything but the call into the plugin is the same whatever API
//! the plugin was built against; each `wit::v0_x::menu` resolves its own `menu-definition` into an
//! [`OpenMenu`] and answers [`Call`]s.

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

use pumpkin_core::entity::player::Player;
use pumpkin_core::server::Server;
use pumpkin_core::world::World;

use crate::plugin::WasmPlugin;

/// Vanilla `AbstractContainerMenu.SLOT_CLICKED_OUTSIDE` aside, slot indexes fit in a byte.
const MAX_SLOT_COUNT: u32 = 256;

/// What the host asks the plugin, with the items as host stacks.
pub enum Call {
    SetItem(u32, Option<ItemStack>),
    MayPlace(u32, ItemStack),
    MayPickup(u32),
    MaxCount(u32, ItemStack),
    QuickMove(u32, ItemStack),
    Closed,
}

pub enum Reply {
    None,
    Allowed(bool),
    Count(u32),
    QuickMove(Vec<QuickMoveStep>),
}

/// A `quick-move-step`: a slot range to try moving a shift-clicked stack into.
pub struct QuickMoveStep {
    pub start: u32,
    pub end: u32,
    pub reverse: bool,
    pub single: bool,
}

/// A `menu-slot`, in the order the client lays them out.
#[derive(Clone, Copy)]
pub enum MenuSlot {
    /// A slot of the viewer's inventory, by inventory index: 0-8 hotbar, 9-35 main.
    Player(u32),
    /// A slot the plugin owns, by the plugin's own index.
    Plugin(u32),
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

/// `menu.update-slot`: sets a plugin slot of every open menu with this handler and menu id.
pub fn update_slot(
    plugin: &Arc<WasmPlugin>,
    handler_id: u32,
    menu_id: u32,
    slot: u32,
    item: ItemStack,
) {
    if let Some(contents) = find_contents(contents_key(plugin, handler_id, menu_id)) {
        contents.set(slot as usize, item);
    }
}

/// `menu.close`: the viewers' next tick sees the menu as no longer valid and closes it.
pub fn close(plugin: &Arc<WasmPlugin>, handler_id: u32, menu_id: u32) {
    if let Some(contents) = find_contents(contents_key(plugin, handler_id, menu_id)) {
        contents.closed.store(true, Ordering::Relaxed);
    }
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
    /// Calls the plugin synchronously, the way block hooks do (see `modded/mod.rs`).
    fn call(&self, call: Call) -> Option<Reply> {
        let plugin = self.plugin.clone();
        let viewer = self.viewer.clone();
        let (handler_id, menu_id) = (self.handler_id, self.menu_id);
        let run = plugin
            .guest
            .menu_call(plugin.current(), handler_id, menu_id, viewer, call);

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

    fn checks_can_use_each_tick(&self) -> bool {
        true
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

pub struct Factory {
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

/// The plugin-slot count of a menu layout, or why the layout is not allowed.
pub fn menu_size(slots: &[MenuSlot]) -> Result<usize, String> {
    if slots.len() as u32 > MAX_SLOT_COUNT {
        return Err(format!(
            "a menu has at most {MAX_SLOT_COUNT} slots, got {}",
            slots.len()
        ));
    }
    let mut size = 0;
    for slot in slots {
        match *slot {
            MenuSlot::Player(index) if index >= 36 => {
                return Err(format!("player slot {index} is not in 0..36"));
            }
            MenuSlot::Plugin(index) => size = size.max(index as usize + 1),
            MenuSlot::Player(_) => {}
        }
    }
    Ok(size)
}

/// A menu as the plugin defined it, checked and resolved against the host state.
pub struct OpenMenu {
    pub menu: Arc<PluginMenu>,
    pub slots: Vec<MenuSlot>,
    pub title: TextComponent,
    pub player: Arc<Player>,
}

impl OpenMenu {
    /// A menu for `player`, its plugin slots set to `items` (padded to `size`). `anchor` is the
    /// block position and the distance the player must stay within.
    #[expect(clippy::too_many_arguments)]
    pub fn new(
        plugin: Arc<WasmPlugin>,
        server: Arc<Server>,
        player: Arc<Player>,
        handler_id: u32,
        menu_id: u32,
        slots: Vec<MenuSlot>,
        size: usize,
        mut items: Vec<ItemStack>,
        title: TextComponent,
        anchor: Option<(BlockPos, f64)>,
    ) -> Self {
        items.resize_with(size, || ItemStack::EMPTY.clone());
        let anchor = anchor.map(|(pos, max_distance)| {
            let world = player.world();
            Anchor {
                block: pumpkin_data::BlockId::from_state_id(world.get_block_state_id(&pos)),
                world: Arc::downgrade(&world),
                pos,
                max_distance_sq: max_distance * max_distance,
            }
        });
        let contents = open_contents(contents_key(&plugin, handler_id, menu_id), items);
        Self {
            menu: Arc::new(PluginMenu {
                plugin,
                handler_id,
                menu_id,
                viewer: player.clone(),
                server,
                size,
                contents,
                anchor,
            }),
            slots,
            title,
            player,
        }
    }

    #[must_use]
    pub fn plugin(&self) -> Arc<WasmPlugin> {
        self.menu.plugin.clone()
    }

    /// Builds the screen handler for a sync id, for menus opened without `open_handled_screen`.
    #[must_use]
    pub fn handler(&self, sync_id: u8) -> SharedScreenHandler {
        Arc::new(StdMutex::new(PluginMenuHandler::new(
            sync_id,
            None,
            self.menu.clone(),
            &self.slots,
            &self.player.inventory,
        )))
    }

    /// The factory for `open_handled_screen`, with the vanilla screen the menu shows as.
    #[must_use]
    pub fn into_factory(self, window_type: WindowType) -> (Arc<Player>, Factory) {
        (
            self.player,
            Factory {
                menu: self.menu,
                window_type,
                slots: self.slots,
                title: self.title,
            },
        )
    }
}
