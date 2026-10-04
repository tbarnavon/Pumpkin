use crate::block::registry::BlockActionResult;
use crate::entity::EntityBase;
use crate::entity::player::Player;
use crate::plugin::loader::wasm::wasm_host::wit::v0_1::modded::PluginItem;
use crate::server::Server;
use pumpkin_data::Block;
use pumpkin_data::BlockDirection;
use pumpkin_data::item::Item;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_util::Hand;
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector3::Vector3;
use rustc_hash::FxHashMap;
use std::sync::Arc;

use super::{ItemBehaviour, ItemMetadata};

pub(crate) const fn should_try_block_placement(result: &BlockActionResult) -> bool {
    matches!(result, BlockActionResult::Pass)
}

pub struct ItemRegistry {
    items: FxHashMap<u16, Arc<dyn ItemBehaviour>>,
    /// Behaviour plugins install at runtime for modded items, indexed by raw item id. Set once
    /// per item, which is what lets `get_pumpkin_item` hand out plain references.
    plugin_items: Box<[std::sync::OnceLock<Arc<dyn ItemBehaviour>>]>,
}

impl Default for ItemRegistry {
    fn default() -> Self {
        Self {
            items: FxHashMap::default(),
            plugin_items: (0..Item::count())
                .map(|_| std::sync::OnceLock::new())
                .collect(),
        }
    }
}

impl ItemRegistry {
    pub fn register<T: ItemBehaviour + ItemMetadata + 'static>(&mut self, item: T) {
        let val = Arc::new(item);
        self.items.reserve(T::ids().len());
        for i in T::ids() {
            self.items.insert(i, val.clone());
        }
    }

    pub fn on_use(&self, stack: &ItemStack, player: &Player, hand: Hand) {
        let (yaw, pitch) = player.rotation();
        self.on_use_with_rotation(stack, player, yaw, pitch, hand);
    }

    pub fn on_use_with_rotation(
        &self,
        stack: &ItemStack,
        player: &Player,
        yaw: f32,
        pitch: f32,
        hand: Hand,
    ) {
        let item = stack.item;
        let cooldown = stack.get_use_cooldown();
        let cooldown_group = cooldown
            .and_then(|c| c.cooldown_group.clone())
            .unwrap_or_else(|| item.registry_key.to_string());

        if player.is_on_cooldown(&cooldown_group) {
            return;
        }

        let pumpkin_item = self.get_pumpkin_item(item.id);
        if let Some(pumpkin_item) = pumpkin_item {
            pumpkin_item.normal_use_with_hand(item, player, yaw, pitch, hand);
        }

        if let Some(cooldown) = cooldown {
            player.start_cooldown(cooldown_group, (cooldown.seconds * 20.0) as i32);
        }
    }

    pub fn on_stopped_using(&self, stack: &ItemStack, player: &Player) {
        if let Some(behaviour) = self.get_pumpkin_item(stack.item.id) {
            behaviour.on_stopped_using(stack, player);
        }
    }

    pub fn on_spear_jab(&self, stack: &ItemStack, player: &Player) {
        if let Some(behaviour) = self.get_pumpkin_item(stack.item.id) {
            behaviour.on_spear_jab(stack, player);
        }
    }

    pub fn on_use_tick(&self, stack: &ItemStack, player: &Player, remaining_use_ticks: i32) {
        if let Some(behaviour) = self.get_pumpkin_item(stack.item.id) {
            behaviour.on_use_tick(stack, player, remaining_use_ticks);
        }
    }

    /// `Item.finishUsingItem`: the stack to put in `hand`, if the item's behaviour replaces it.
    pub fn finish_using(
        &self,
        stack: &ItemStack,
        player: &Player,
        hand: Hand,
    ) -> Option<ItemStack> {
        self.get_pumpkin_item(stack.item.id)
            .and_then(|behaviour| behaviour.finish_using(stack, player, hand))
    }

    /// `Item.useOnRelease`.
    #[must_use]
    pub fn use_on_release(&self, stack: &ItemStack) -> bool {
        self.get_pumpkin_item(stack.item.id)
            .is_some_and(|behaviour| behaviour.use_on_release())
    }

    /// The use of `stack` ended (`LivingEntity.stopUsingItem`).
    pub fn on_use_ended(&self, stack: &ItemStack, player: &Player, hand: Hand, remaining: i32) {
        if let Some(behaviour) = self.get_pumpkin_item(stack.item.id) {
            behaviour.on_use_ended(stack, player, hand, remaining);
        }
    }

    /// Returns the item's use duration in ticks, as defined by its registered behaviour.
    /// Returns `None` if the item has no registered behaviour or its duration is 0.
    #[must_use]
    pub fn get_use_duration(&self, item_id: u16) -> Option<i32> {
        self.get_pumpkin_item(item_id)
            .map(|b| b.get_use_duration())
            .filter(|&d| d > 0)
    }

    #[expect(clippy::too_many_arguments)]
    pub fn use_on_block(
        &self,
        stack: &mut ItemStack,
        player: &Player,
        location: BlockPos,
        face: BlockDirection,
        cursor_pos: Vector3<f32>,
        block: &Block,
        server: &Server,
    ) -> BlockActionResult {
        let cooldown = stack.get_use_cooldown().cloned();
        let cooldown_group = cooldown
            .as_ref()
            .and_then(|c| c.cooldown_group.clone())
            .unwrap_or_else(|| stack.item.registry_key.to_string());

        if player.is_on_cooldown(&cooldown_group) {
            return BlockActionResult::Pass;
        }

        let pumpkin_item = self.get_pumpkin_item(stack.item.id);
        let result = pumpkin_item.map_or(BlockActionResult::Pass, |pumpkin_item| {
            pumpkin_item.use_on_block(stack, player, location, face, cursor_pos, block, server)
        });

        if let Some(cooldown) = cooldown {
            player.start_cooldown(cooldown_group, (cooldown.seconds * 20.0) as i32);
        }

        result
    }

    pub fn use_on_entity(
        &self,
        stack: &mut ItemStack,
        player: &Player,
        entity: Arc<dyn EntityBase>,
    ) {
        let cooldown = stack.get_use_cooldown().cloned();
        let cooldown_group = cooldown
            .as_ref()
            .and_then(|c| c.cooldown_group.clone())
            .unwrap_or_else(|| stack.item.registry_key.to_string());

        if player.is_on_cooldown(&cooldown_group) {
            return;
        }

        let pumpkin_item = self.get_pumpkin_item(stack.item.id);
        if let Some(pumpkin_item) = pumpkin_item {
            pumpkin_item.use_on_entity(stack, player, entity);
        }

        if let Some(cooldown) = cooldown {
            player.start_cooldown(cooldown_group, (cooldown.seconds * 20.0) as i32);
        }
    }

    pub fn can_mine(&self, item: &Item, player: &Player) -> bool {
        let pumpkin_block = self.get_pumpkin_item(item.id);
        if let Some(pumpkin_block) = pumpkin_block {
            return pumpkin_block.can_mine(player);
        }
        true
    }

    #[must_use]
    pub fn get_pumpkin_item(&self, item: u16) -> Option<&Arc<dyn ItemBehaviour>> {
        self.items.get(&item).or_else(|| {
            self.plugin_items
                .get(usize::from(item))
                .and_then(std::sync::OnceLock::get)
        })
    }

    /// Installs a plugin's behaviour for a modded item. Each item can get one behaviour, and
    /// only items without a native one.
    pub fn register_plugin_item(
        &self,
        item: &'static Item,
        behaviour: Arc<dyn ItemBehaviour>,
    ) -> Result<(), String> {
        if item.is_vanilla() {
            return Err(format!("{} is a vanilla item", item.namespaced_name()));
        }
        let slot = self
            .plugin_items
            .get(usize::from(item.id))
            .ok_or_else(|| format!("{} is not registered", item.namespaced_name()))?;
        if let Some(existing) = slot.get() {
            // A restarted plugin registering the same handler again keeps the existing one.
            return if PluginItem::same_handler(existing.as_ref(), behaviour.as_ref()) {
                Ok(())
            } else {
                Err(format!(
                    "{} already has plugin hooks",
                    item.namespaced_name()
                ))
            };
        }
        slot.set(behaviour)
            .map_err(|_| format!("{} already has plugin hooks", item.namespaced_name()))
    }
}

#[cfg(test)]
mod tests {
    use super::should_try_block_placement;
    use crate::block::registry::BlockActionResult;

    #[test]
    fn block_placement_only_follows_pass() {
        assert!(should_try_block_placement(&BlockActionResult::Pass));

        for result in [
            BlockActionResult::Success,
            BlockActionResult::SuccessServer,
            BlockActionResult::Consume,
            BlockActionResult::Fail,
            BlockActionResult::PassToDefaultBlockAction,
        ] {
            assert!(!should_try_block_placement(&result));
        }
    }
}
