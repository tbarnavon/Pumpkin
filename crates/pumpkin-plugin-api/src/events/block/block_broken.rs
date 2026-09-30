use super::super::FromIntoEvent;
use crate::wit::pumpkin::plugin::event::{BlockBrokenEventData, Event, EventType};

/// A block was broken, after it left the world (Fabric `PlayerBlockBreakEvents.AFTER`).
pub struct BlockBrokenEvent;
impl FromIntoEvent for BlockBrokenEvent {
    const EVENT_TYPE: EventType = EventType::BlockBrokenEvent;
    type Data = BlockBrokenEventData;

    fn data_from_event(event: Event) -> Self::Data {
        match event {
            Event::BlockBrokenEvent(data) => data,
            _ => panic!("unexpected event"),
        }
    }

    fn data_into_event(data: Self::Data) -> Event {
        Event::BlockBrokenEvent(data)
    }
}
