use super::super::FromIntoEvent;
use crate::wit::pumpkin::plugin::event::{BlockEntityUnloadEventData, Event, EventType};

/// A block entity left its world, removed or unloaded with its chunk (Fabric `ServerBlockEntityEvents.BLOCK_ENTITY_UNLOAD`).
pub struct BlockEntityUnloadEvent;
impl FromIntoEvent for BlockEntityUnloadEvent {
    const EVENT_TYPE: EventType = EventType::BlockEntityUnloadEvent;
    type Data = BlockEntityUnloadEventData;

    fn data_from_event(event: Event) -> Self::Data {
        match event {
            Event::BlockEntityUnloadEvent(data) => data,
            _ => panic!("unexpected event"),
        }
    }

    fn data_into_event(data: Self::Data) -> Event {
        Event::BlockEntityUnloadEvent(data)
    }
}
