use super::super::FromIntoEvent;
use crate::wit::pumpkin::plugin::event::{Event, EventType, PlayerLoginQueryResponseEventData};

/// A client answered a login query registered with `context.register-login-query` (Fabric `ServerLoginNetworking` receivers). Cancel to disconnect the client.
pub struct PlayerLoginQueryResponseEvent;
impl FromIntoEvent for PlayerLoginQueryResponseEvent {
    const EVENT_TYPE: EventType = EventType::PlayerLoginQueryResponseEvent;
    type Data = PlayerLoginQueryResponseEventData;

    fn data_from_event(event: Event) -> Self::Data {
        match event {
            Event::PlayerLoginQueryResponseEvent(data) => data,
            _ => panic!("unexpected event"),
        }
    }

    fn data_into_event(data: Self::Data) -> Event {
        Event::PlayerLoginQueryResponseEvent(data)
    }
}
