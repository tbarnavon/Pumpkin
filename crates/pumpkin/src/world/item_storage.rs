//! Item storages that plugins attach to their own block entities, like a mod's
//! `ItemStorage.SIDED` (Fabric Transfer API).
//!
//! The host keeps the slots (in the block entity's chunk NBT, under [`NBT_KEY`]) so hoppers can
//! move items without calling the plugin. The plugin learns what changed through one
//! `ItemStorageChangedEvent` per world tick, and reads the slots back with
//! `world.get-item-storage`.

use std::any::Any;
use std::sync::{Arc, Mutex};

use pumpkin_data::item_stack::ItemStack;
use pumpkin_inventory::Clearable;
use pumpkin_inventory::inventory::Inventory;
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_nbt::tag::NbtTag;
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::chunk::io::Dirtiable;

use super::World;

/// The block-entity NBT key the slots are saved under. Stripped from the data plugins see.
pub const NBT_KEY: &str = "PumpkinItemStorage";

/// A slot with no stack limit.
pub const UNLIMITED: u32 = u32::MAX;

#[derive(Clone)]
pub struct StorageSlot {
    /// The stored item (count 1), or empty.
    pub item: ItemStack,
    pub count: u32,
    /// Capacity in stacks of the slot's item, or [`UNLIMITED`].
    pub max_stacks: u32,
    pub insert: bool,
    pub extract: bool,
    /// An empty slot takes any item.
    pub accept_new: bool,
    /// The item stays in the slot when its count reaches zero.
    pub keep_item: bool,
    /// Items past capacity are accepted and destroyed.
    pub void_overflow: bool,
}

impl StorageSlot {
    fn capacity(&self, item: &ItemStack) -> u32 {
        if self.max_stacks == UNLIMITED {
            return i32::MAX as u32;
        }
        u32::from(item.get_max_stack_size())
            .saturating_mul(self.max_stacks)
            .min(i32::MAX as u32)
    }

    const fn has_item(&self) -> bool {
        !self.item.is_empty()
    }

    /// The stack a hopper sees: at most one stack, and one item short of full while there is
    /// room, so hoppers keep adding.
    fn view(&self) -> ItemStack {
        if !self.has_item() || self.count == 0 {
            return ItemStack::EMPTY.clone();
        }
        let max = u32::from(self.item.get_max_stack_size());
        let room = self.void_overflow || self.count < self.capacity(&self.item);
        let shown = if self.count >= max && room && max > 1 {
            max - 1
        } else {
            self.count.min(max)
        };
        self.item.copy_with_count(shown as u8)
    }

    fn accepts(&self, stack: &ItemStack) -> bool {
        if !self.insert || stack.is_empty() {
            return false;
        }
        if self.has_item() {
            self.item.are_items_and_components_equal(stack)
                && (self.void_overflow || self.count < self.capacity(&self.item))
        } else {
            self.accept_new
        }
    }

    /// Applies a hopper's write to the view as a change of the real count.
    fn apply(&mut self, stack: &ItemStack) {
        let shown = u32::from(self.view().item_count);
        if stack.is_empty() {
            self.count = self.count.saturating_sub(shown);
        } else if !self.has_item() {
            self.item = stack.copy_with_count(1);
            self.count = u32::from(stack.item_count).min(self.capacity(stack));
        } else {
            let target = u32::from(stack.item_count);
            if target >= shown {
                self.count = self
                    .count
                    .saturating_add(target - shown)
                    .min(self.capacity(&self.item));
            } else {
                self.count = self.count.saturating_sub(shown - target);
            }
        }
        if self.count == 0 && !self.keep_item {
            self.item = ItemStack::EMPTY.clone();
        }
    }

    const FLAG_INSERT: u8 = 1;
    const FLAG_EXTRACT: u8 = 2;
    const FLAG_ACCEPT_NEW: u8 = 4;
    const FLAG_KEEP_ITEM: u8 = 8;
    const FLAG_VOID: u8 = 16;

    fn write(&self) -> NbtCompound {
        let mut nbt = NbtCompound::new();
        if self.has_item() {
            let mut item = NbtCompound::new();
            self.item.write_item_stack(&mut item);
            nbt.put_compound("item", item);
        }
        nbt.put_int("count", self.count as i32);
        nbt.put_int("max_stacks", self.max_stacks as i32);
        let flags = [
            (self.insert, Self::FLAG_INSERT),
            (self.extract, Self::FLAG_EXTRACT),
            (self.accept_new, Self::FLAG_ACCEPT_NEW),
            (self.keep_item, Self::FLAG_KEEP_ITEM),
            (self.void_overflow, Self::FLAG_VOID),
        ]
        .iter()
        .filter(|(on, _)| *on)
        .fold(0u8, |bits, (_, bit)| bits | bit);
        nbt.put_byte("flags", flags as i8);
        nbt
    }

    fn read(nbt: &NbtCompound) -> Self {
        let flags = nbt.get_byte("flags").unwrap_or(0) as u8;
        Self {
            item: nbt
                .get_compound("item")
                .and_then(ItemStack::read_item_stack)
                .map_or_else(|| ItemStack::EMPTY.clone(), |item| item.copy_with_count(1)),
            count: nbt.get_int("count").unwrap_or(0) as u32,
            max_stacks: nbt.get_int("max_stacks").unwrap_or(0) as u32,
            insert: flags & Self::FLAG_INSERT != 0,
            extract: flags & Self::FLAG_EXTRACT != 0,
            accept_new: flags & Self::FLAG_ACCEPT_NEW != 0,
            keep_item: flags & Self::FLAG_KEEP_ITEM != 0,
            void_overflow: flags & Self::FLAG_VOID != 0,
        }
    }
}

#[must_use]
pub fn write_slots(slots: &[StorageSlot]) -> NbtTag {
    NbtTag::List(
        slots
            .iter()
            .map(|slot| NbtTag::Compound(slot.write()))
            .collect(),
    )
}

#[must_use]
pub fn read_slots(nbt: &NbtCompound) -> Option<Vec<StorageSlot>> {
    let list = nbt.get_list(NBT_KEY)?;
    Some(
        list.iter()
            .filter_map(NbtTag::extract_compound)
            .map(StorageSlot::read)
            .collect(),
    )
}

/// A plugin block's storage, seen by hoppers as an inventory.
pub struct PluginItemStorage {
    world: std::sync::Weak<World>,
    pos: BlockPos,
    slots: Mutex<Vec<StorageSlot>>,
}

impl PluginItemStorage {
    fn changed(&self) {
        let Some(world) = self.world.upgrade() else {
            return;
        };
        let slots = self
            .slots
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clone();
        world.write_item_storage_nbt(&self.pos, &slots);
        world.queue_item_storage_change(self.pos);
    }
}

impl Clearable for PluginItemStorage {
    fn clear(&self) {}
}

impl Inventory for PluginItemStorage {
    fn size(&self) -> usize {
        self.slots
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .len()
    }

    fn is_empty(&self) -> bool {
        self.slots
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .iter()
            .all(|slot| slot.count == 0)
    }

    fn get_stack(&self, slot: usize) -> ItemStack {
        self.slots
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .get(slot)
            .map_or_else(|| ItemStack::EMPTY.clone(), StorageSlot::view)
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
        {
            let mut slots = self
                .slots
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            let Some(entry) = slots.get_mut(slot) else {
                return;
            };
            entry.apply(&stack);
        };
        self.changed();
    }

    fn is_valid_slot_for(&self, slot: usize, stack: &ItemStack) -> bool {
        self.slots
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .get(slot)
            .is_some_and(|entry| entry.accepts(stack))
    }

    fn can_transfer_to(&self, _hopper: &dyn Inventory, slot: usize, _stack: &ItemStack) -> bool {
        self.slots
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .get(slot)
            .is_some_and(|entry| entry.extract)
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl World {
    /// The stored NBT of a block entity Pumpkin has no implementation of.
    pub(crate) fn pending_block_entity_nbt(&self, pos: &BlockPos) -> Option<NbtCompound> {
        self.level
            .read_chunk_sync(&pos.chunk_position(), |chunk| {
                chunk
                    .pending_block_entities
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner)
                    .get(pos)
                    .cloned()
            })
            .flatten()
    }

    /// The plugin item storage at `pos`, if its block entity has one.
    pub fn item_storage(self: &Arc<Self>, pos: &BlockPos) -> Option<Arc<PluginItemStorage>> {
        if let Some(storage) = self.item_storages.get(pos) {
            return Some(storage.clone());
        }
        let slots = read_slots(&self.pending_block_entity_nbt(pos)?)?;
        let storage = Arc::new(PluginItemStorage {
            world: Arc::downgrade(self),
            pos: *pos,
            slots: Mutex::new(slots),
        });
        self.item_storages.insert(*pos, storage.clone());
        Some(storage)
    }

    /// The inventory hoppers and droppers use at `pos`: a block entity's own, or a plugin
    /// block's item storage.
    pub fn get_inventory_at(self: &Arc<Self>, pos: &BlockPos) -> Option<Arc<dyn Inventory>> {
        if let Some(entity) = self.get_block_entity(pos) {
            return entity.get_inventory();
        }
        self.item_storage(pos)
            .map(|storage| storage as Arc<dyn Inventory>)
    }

    /// Replaces the item storage of the plugin block entity at `pos`.
    pub fn set_item_storage(&self, pos: &BlockPos, slots: &[StorageSlot]) -> Result<(), String> {
        if self.pending_block_entity_nbt(pos).is_none() {
            return Err("no plugin block entity at this position".to_string());
        }
        self.write_item_storage_nbt(pos, slots);
        if let Some(storage) = self.item_storages.get(pos) {
            *storage
                .slots
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner) = slots.to_vec();
        }
        Ok(())
    }

    pub fn get_item_storage(&self, pos: &BlockPos) -> Option<Vec<StorageSlot>> {
        if let Some(storage) = self.item_storages.get(pos) {
            return Some(
                storage
                    .slots
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner)
                    .clone(),
            );
        }
        read_slots(&self.pending_block_entity_nbt(pos)?)
    }

    fn write_item_storage_nbt(&self, pos: &BlockPos, slots: &[StorageSlot]) {
        self.level.read_chunk_sync(&pos.chunk_position(), |chunk| {
            let mut pending = chunk
                .pending_block_entities
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            if let Some(nbt) = pending.get_mut(pos) {
                nbt.put(NBT_KEY, write_slots(slots));
                drop(pending);
                chunk.mark_dirty(true);
            }
        });
    }

    /// Forgets the cached storage at `pos` (the block entity is gone or was rewritten).
    pub(crate) fn forget_item_storage(&self, pos: &BlockPos) {
        self.item_storages.remove(pos);
    }

    fn queue_item_storage_change(&self, pos: BlockPos) {
        let mut changes = self
            .item_storage_changes
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if !changes.contains(&pos) {
            changes.push(pos);
        }
    }

    /// Tells plugins which item storages changed during the last tick.
    pub(crate) fn flush_item_storage_changes(
        self: &Arc<Self>,
        server: &Arc<crate::server::Server>,
    ) {
        let positions = std::mem::take(
            &mut *self
                .item_storage_changes
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner),
        );
        if positions.is_empty() {
            return;
        }
        let mut event =
            crate::plugin::api::events::inventory::item_storage_changed::ItemStorageChangedEvent {
                world: self.clone(),
                positions,
            };
        server.plugin_manager.fire_blocking(server, &mut event);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pumpkin_data::item::Item;

    fn slot(item: &'static Item, count: u32, max_stacks: u32) -> StorageSlot {
        StorageSlot {
            item: ItemStack::new(1, item),
            count,
            max_stacks,
            insert: true,
            extract: true,
            accept_new: true,
            keep_item: false,
            void_overflow: false,
        }
    }

    #[test]
    fn big_slot_shows_one_short_of_a_stack_while_there_is_room() {
        let slot = slot(&Item::COBBLESTONE, 200, 32);
        assert_eq!(slot.view().item_count, 63);
    }

    #[test]
    fn full_slot_shows_a_full_stack() {
        let slot = slot(&Item::COBBLESTONE, 64 * 2, 2);
        assert_eq!(slot.view().item_count, 64);
        assert!(!slot.accepts(&ItemStack::new(1, &Item::COBBLESTONE)));
    }

    #[test]
    fn hopper_insert_adds_one_to_the_real_count() {
        let mut slot = slot(&Item::COBBLESTONE, 200, 32);
        let mut shown = slot.view();
        shown.item_count += 1;
        slot.apply(&shown);
        assert_eq!(slot.count, 201);
    }

    #[test]
    fn hopper_extract_removes_one_and_clears_the_last() {
        let mut slot = slot(&Item::COBBLESTONE, 1, 32);
        let mut shown = slot.view();
        let _ = shown.split(1);
        slot.apply(&shown);
        assert_eq!(slot.count, 0);
        assert!(slot.item.is_empty());
    }

    #[test]
    fn void_slot_swallows_past_capacity() {
        let mut slot = slot(&Item::COBBLESTONE, 64, 1);
        slot.void_overflow = true;
        assert!(slot.accepts(&ItemStack::new(1, &Item::COBBLESTONE)));
        let mut shown = slot.view();
        shown.item_count += 1;
        slot.apply(&shown);
        assert_eq!(slot.count, 64);
    }

    #[test]
    fn slots_round_trip_through_nbt() {
        let slots = vec![
            slot(&Item::COBBLESTONE, 300, 32),
            StorageSlot {
                item: ItemStack::EMPTY.clone(),
                count: 0,
                max_stacks: UNLIMITED,
                insert: true,
                extract: false,
                accept_new: false,
                keep_item: true,
                void_overflow: true,
            },
        ];
        let mut nbt = NbtCompound::new();
        nbt.put(NBT_KEY, write_slots(&slots));
        let read = read_slots(&nbt).expect("slots are saved");
        assert_eq!(write_slots(&read), write_slots(&slots));
    }
}
