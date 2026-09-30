use super::super::FromIntoEvent;
use crate::wit::pumpkin::plugin::event::{Event, EventType, PlayerStartTrackingEventData};

/// An entity became visible to a player's client (Fabric `EntityTrackingEvents.START_TRACKING`).
pub struct PlayerStartTrackingEvent;
impl FromIntoEvent for PlayerStartTrackingEvent {
    const EVENT_TYPE: EventType = EventType::PlayerStartTrackingEvent;
    type Data = PlayerStartTrackingEventData;

    fn data_from_event(event: Event) -> Self::Data {
        match event {
            Event::PlayerStartTrackingEvent(data) => data,
            _ => panic!("unexpected event"),
        }
    }

    fn data_into_event(data: Self::Data) -> Event {
        Event::PlayerStartTrackingEvent(data)
    }
}
