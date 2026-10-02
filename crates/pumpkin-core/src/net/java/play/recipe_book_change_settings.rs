#[allow(clippy::wildcard_imports)]
use super::*;

impl JavaClient {
    pub fn handle_recipe_book_change_settings(
        &self,
        server: &Arc<Server>,
        player: &Arc<Player>,
        packet: &SRecipeBookChangeSettings,
    ) {
        let mut event = crate::plugin::api::events::player::player_recipe_book_settings_change::PlayerRecipeBookSettingsChangeEvent::new(
            player.clone(),
            format!("{:?}", packet.book_type),
            packet.is_open,
            packet.is_filtering,
        );
        server.plugin_manager.fire_blocking(server, &mut event);
        if event.cancelled {
            return;
        }
        // `ServerRecipeBook.setBookSetting`
        if let Ok(book) = usize::try_from(packet.book_type.0) {
            player.set_recipe_book_flags(book, packet.is_open, packet.is_filtering);
        }
    }
}
