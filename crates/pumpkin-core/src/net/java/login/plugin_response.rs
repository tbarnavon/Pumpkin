#[allow(clippy::wildcard_imports)]
use super::*;

impl PendingConnection {
    pub async fn handle_plugin_response(
        &mut self,
        server: &Arc<Server>,
        plugin_response: SLoginPluginResponse,
    ) -> Option<PacketHandlerResult> {
        debug!("Handling plugin");
        if let Some(index) = self
            .login_queries
            .iter()
            .position(|(id, _)| *id == plugin_response.message_id.0)
        {
            let (_, channel) = self.login_queries.remove(index);
            let profile = self.gameprofile.clone()?;
            let mut event = crate::plugin::api::events::player::player_login_query_response::PlayerLoginQueryResponseEvent {
                player_name: profile.name.clone(),
                player_uuid: profile.id,
                channel,
                understood: plugin_response.data.is_some(),
                data: plugin_response.data.map(Vec::from),
                cancelled: false,
            };
            server.plugin_manager.fire(server, &mut event).await;
            if event.cancelled {
                self.kick(TextComponent::text("Disconnected")).await;
                return Some(PacketHandlerResult::Stop);
            }
            if self.login_queries.is_empty() {
                self.send_login_success(&profile).await;
            }
            return None;
        }
        let proxy_config = &server.advanced_config.networking.proxy;
        if proxy_config.vine.enabled {
            let expected_challenge = self.vine_challenge.take();
            match vine::receive_vine_plugin_response(
                self.address.port(),
                &proxy_config.vine,
                plugin_response,
                expected_challenge,
            ) {
                Ok((profile, new_address)) => {
                    self.gameprofile = Some(profile.clone());
                    self.address = new_address;
                    self.finish_login(server, &profile).await
                }
                Err(error) => {
                    self.kick(TextComponent::text(error.to_string())).await;
                    Some(PacketHandlerResult::Stop)
                }
            }
        } else if proxy_config.velocity.enabled {
            match velocity::receive_velocity_plugin_response(
                self.address.port(),
                &proxy_config.velocity,
                plugin_response,
            ) {
                Ok((profile, new_address)) => {
                    self.gameprofile = Some(profile.clone());
                    self.address = new_address;
                    self.finish_login(server, &profile).await
                }
                Err(error) => {
                    self.kick(TextComponent::text(error.to_string())).await;
                    Some(PacketHandlerResult::Stop)
                }
            }
        } else {
            None
        }
    }
}
