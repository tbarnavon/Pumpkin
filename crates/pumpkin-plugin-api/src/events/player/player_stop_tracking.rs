use super::super::FromIntoEvent;
use crate::wit::pumpkin::plugin::event::{Event, EventType, PlayerStopTrackingEventData};

/// An entity stopped being visible to a player's client (Fabric `EntityTrackingEvents.STOP_TRACKING`).
pub struct PlayerStopTrackingEvent;
impl FromIntoEvent for PlayerStopTrackingEvent {
    const EVENT_TYPE: EventType = EventType::PlayerStopTrackingEvent;
    type Data = PlayerStopTrackingEventData;

    fn data_from_event(event: Event) -> Self::Data {
        match event {
            Event::PlayerStopTrackingEvent(data) => data,
            _ => panic!("unexpected event"),
        }
    }

    fn data_into_event(data: Self::Data) -> Event {
        Event::PlayerStopTrackingEvent(data)
    }
}
