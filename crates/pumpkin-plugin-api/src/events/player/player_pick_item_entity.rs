use super::super::FromIntoEvent;
use crate::wit::pumpkin::plugin::event::{Event, EventType, PlayerPickItemEntityEventData};

/// A player middle-clicks an entity (Fabric `PlayerPickItemEvents.ENTITY`). Set `item` to give that stack instead of the spawn egg; cancel to give nothing.
pub struct PlayerPickItemEntityEvent;
impl FromIntoEvent for PlayerPickItemEntityEvent {
    const EVENT_TYPE: EventType = EventType::PlayerPickItemEntityEvent;
    type Data = PlayerPickItemEntityEventData;

    fn data_from_event(event: Event) -> Self::Data {
        match event {
            Event::PlayerPickItemEntityEvent(data) => data,
            _ => panic!("unexpected event"),
        }
    }

    fn data_into_event(data: Self::Data) -> Event {
        Event::PlayerPickItemEntityEvent(data)
    }
}
