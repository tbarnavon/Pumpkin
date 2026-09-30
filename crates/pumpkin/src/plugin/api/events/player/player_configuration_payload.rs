use pumpkin_macros::{Event, cancellable};

/// A client sent a custom payload during the configuration phase (Fabric `ServerConfigurationNetworking` receivers). Cancel to disconnect the client.
#[cancellable]
#[derive(Event, Clone)]
pub struct PlayerConfigurationPayloadEvent {
    pub player_name: String,
    pub player_uuid: uuid::Uuid,
    pub channel: String,
    pub data: Vec<u8>,
}
