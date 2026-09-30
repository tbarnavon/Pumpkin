use super::super::FromIntoEvent;
use crate::wit::pumpkin::plugin::event::{BlockBreakCanceledEventData, Event, EventType};

/// A `block-break-event` was cancelled (Fabric `PlayerBlockBreakEvents.CANCELED`).
pub struct BlockBreakCanceledEvent;
impl FromIntoEvent for BlockBreakCanceledEvent {
    const EVENT_TYPE: EventType = EventType::BlockBreakCanceledEvent;
    type Data = BlockBreakCanceledEventData;

    fn data_from_event(event: Event) -> Self::Data {
        match event {
            Event::BlockBreakCanceledEvent(data) => data,
            _ => panic!("unexpected event"),
        }
    }

    fn data_into_event(data: Self::Data) -> Event {
        Event::BlockBreakCanceledEvent(data)
    }
}
