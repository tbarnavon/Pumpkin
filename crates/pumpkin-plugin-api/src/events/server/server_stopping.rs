use super::super::FromIntoEvent;
use crate::wit::pumpkin::plugin::event::{Event, EventType, ServerStoppingEventData};

/// The server is stopping: players are still online and worlds not yet saved (Fabric `ServerLifecycleEvents.SERVER_STOPPING`). Plugins unload right after; `on-unload` is the last call.
pub struct ServerStoppingEvent;
impl FromIntoEvent for ServerStoppingEvent {
    const EVENT_TYPE: EventType = EventType::ServerStoppingEvent;
    type Data = ServerStoppingEventData;

    fn data_from_event(event: Event) -> Self::Data {
        match event {
            Event::ServerStoppingEvent(data) => data,
            _ => panic!("unexpected event"),
        }
    }

    fn data_into_event(data: Self::Data) -> Event {
        Event::ServerStoppingEvent(data)
    }
}
