use pumpkin_macros::Event;

/// The server is stopping: players are still online and worlds not yet saved (Fabric `ServerLifecycleEvents.SERVER_STOPPING`). Plugins unload right after; `on-unload` is the last call.
#[derive(Event, Clone)]
pub struct ServerStoppingEvent {
    pub player_count: u32,
}
