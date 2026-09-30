use super::super::FromIntoEvent;
use crate::wit::pumpkin::plugin::event::{BlockEntityLoadEventData, Event, EventType};

/// A block entity was added to its world, placed or read from its chunk (Fabric `ServerBlockEntityEvents.BLOCK_ENTITY_LOAD`). Pumpkin creates block entities the first time they are used.
pub struct BlockEntityLoadEvent;
impl FromIntoEvent for BlockEntityLoadEvent {
    const EVENT_TYPE: EventType = EventType::BlockEntityLoadEvent;
    type Data = BlockEntityLoadEventData;

    fn data_from_event(event: Event) -> Self::Data {
        match event {
            Event::BlockEntityLoadEvent(data) => data,
            _ => panic!("unexpected event"),
        }
    }

    fn data_into_event(data: Self::Data) -> Event {
        Event::BlockEntityLoadEvent(data)
    }
}
