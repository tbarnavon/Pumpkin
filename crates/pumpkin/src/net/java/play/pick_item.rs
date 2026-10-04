#[allow(clippy::wildcard_imports)]
use super::*;

use crate::plugin::api::events::player::player_pick_item_block::PlayerPickItemBlockEvent;
use crate::plugin::api::events::player::player_pick_item_entity::PlayerPickItemEntityEvent;

impl JavaClient {
    pub fn handle_pick_item_from_block(
        &self,
        player: &Arc<Player>,
        pick_item: &SPickItemFromBlock,
    ) {
        if !player.can_interact_with_block_at(&pick_item.pos, 1.0) {
            return;
        }

        let world = player.world();
        let state_id = world.get_block_state_id(&pick_item.pos);
        let block = state_id.to_block();

        // Blocks without an item (tall seagrass...) pick nothing by default.
        let default = (block.item_id != 0)
            .then(|| Item::from_id(block.item_id))
            .flatten()
            .map(|item| ItemStack::new(1, item));

        let mut event = PlayerPickItemBlockEvent {
            player: player.clone(),
            block_position: pick_item.pos,
            state_id: state_id.as_u16(),
            include_data: pick_item.include_data,
            item: None,
            cancelled: false,
        };
        if let Some(server) = world.server.upgrade() {
            server.plugin_manager.fire_blocking(&server, &mut event);
        }
        if event.cancelled {
            return;
        }
        if let Some(stack) = event.item.or(default) {
            Self::pick(player, stack);
        }
    }

    pub fn handle_pick_item_from_entity(
        &self,
        player: &Arc<Player>,
        pick_item: &SPickItemFromEntity,
    ) {
        use pumpkin_data::entity::{entity_from_egg, spawn_egg_ids};

        let world = player.world();
        let Some(target) = world.get_entity_by_id(pick_item.id.0) else {
            return;
        };

        let p_eye = player.get_entity().get_eye_pos();
        let t_eye = target.get_eye_pos();
        let dx = p_eye.x - t_eye.x;
        let dy = p_eye.y - t_eye.y;
        let dz = p_eye.z - t_eye.z;
        if dx * dx + dy * dy + dz * dz > 64.0 {
            return;
        }

        let target_type = target.get_entity().entity_type;
        let default = spawn_egg_ids()
            .into_iter()
            .find(|&egg_id| entity_from_egg(egg_id).is_some_and(|et| et.id == target_type.id))
            .and_then(Item::from_id)
            .map(|item| ItemStack::new(1, item));

        let mut event = PlayerPickItemEntityEvent {
            player: player.clone(),
            entity_id: pick_item.id.0,
            entity_type: format!("minecraft:{}", target_type.resource_name),
            include_data: pick_item.include_data,
            item: None,
            cancelled: false,
        };
        if let Some(server) = world.server.upgrade() {
            server.plugin_manager.fire_blocking(&server, &mut event);
        }
        if event.cancelled {
            return;
        }
        if let Some(stack) = event.item.or(default) {
            Self::pick(player, stack);
        }
    }

    /// 1.21.1's middle click: swaps an inventory slot into the hotbar
    /// (`ServerGamePacketListenerImpl.handlePickItem`, `Inventory.pickSlot`).
    pub fn handle_pick_item(
        &self,
        player: &Arc<Player>,
        pick_item: &pumpkin_protocol::java::server::play::SPickItem,
    ) {
        // Vanilla trusts the slot; only main inventory slots can be picked.
        let Ok(slot) = usize::try_from(pick_item.slot.0) else {
            return;
        };
        if slot >= PlayerInventory::MAIN_SIZE {
            return;
        }
        player.inventory.swap_slot_with_hotbar(slot);

        let selected = player.inventory.get_selected_slot();
        for changed in [usize::from(selected), slot] {
            let stack = player.inventory.get_slot(changed);
            player.try_send_client_packet(
                &pumpkin_protocol::java::client::play::CSetContainerSlot::new(
                    -2,
                    0,
                    changed as i16,
                    &pumpkin_protocol::codec::item_stack_seralizer::ItemStackSerializer::from(
                        stack,
                    ),
                ),
            );
        }
        player.try_send_client_packet(&CSetSelectedSlot::new(selected as i8));
    }

    /// Selects the picked stack if the player has it, or puts it in the hotbar in creative.
    fn pick(player: &Arc<Player>, stack: ItemStack) {
        let slot_with_stack = player.inventory().get_slot_with_stack(&stack);

        if slot_with_stack != -1 {
            if PlayerInventory::is_valid_hotbar_index(slot_with_stack as usize) {
                player.inventory.set_selected_slot(slot_with_stack as u8);
            } else {
                player
                    .inventory
                    .swap_slot_with_hotbar(slot_with_stack as usize);
            }
        } else if player.gamemode.load() == GameMode::Creative {
            player.inventory.swap_stack_with_hotbar(stack);
        }

        player.try_send_client_packet(&CSetSelectedSlot::new(
            player.inventory.get_selected_slot() as i8
        ));
        player
            .player_screen_handler
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .send_content_updates();
    }
}
