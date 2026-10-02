use pumpkin_macros::{Event, cancellable};

/// A client answered a login query registered with `context.register-login-query` (Fabric `ServerLoginNetworking` receivers). Cancel to disconnect the client.
#[cancellable]
#[derive(Event, Clone)]
pub struct PlayerLoginQueryResponseEvent {
    pub player_name: String,
    pub player_uuid: uuid::Uuid,
    pub channel: String,
    pub understood: bool,
    pub data: Option<Vec<u8>>,
}
