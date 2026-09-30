use super::super::FromIntoEvent;
use crate::wit::pumpkin::plugin::event::{Event, EventType, PlayerPickItemBlockEventData};

/// A player middle-clicks a block (Fabric `PlayerPickItemEvents.BLOCK`). Set `item` to give that stack instead of the block's item; cancel to give nothing.
pub struct PlayerPickItemBlockEvent;
impl FromIntoEvent for PlayerPickItemBlockEvent {
    const EVENT_TYPE: EventType = EventType::PlayerPickItemBlockEvent;
    type Data = PlayerPickItemBlockEventData;

    fn data_from_event(event: Event) -> Self::Data {
        match event {
            Event::PlayerPickItemBlockEvent(data) => data,
            _ => panic!("unexpected event"),
        }
    }

    fn data_into_event(data: Self::Data) -> Event {
        Event::PlayerPickItemBlockEvent(data)
    }
}
