use super::super::FromIntoEvent;
use crate::wit::pumpkin::plugin::event::{Event, EventType, PlayerConfigurationPayloadEventData};

/// A client sent a custom payload during the configuration phase (Fabric `ServerConfigurationNetworking` receivers). Cancel to disconnect the client.
pub struct PlayerConfigurationPayloadEvent;
impl FromIntoEvent for PlayerConfigurationPayloadEvent {
    const EVENT_TYPE: EventType = EventType::PlayerConfigurationPayloadEvent;
    type Data = PlayerConfigurationPayloadEventData;

    fn data_from_event(event: Event) -> Self::Data {
        match event {
            Event::PlayerConfigurationPayloadEvent(data) => data,
            _ => panic!("unexpected event"),
        }
    }

    fn data_into_event(data: Self::Data) -> Event {
        Event::PlayerConfigurationPayloadEvent(data)
    }
}
