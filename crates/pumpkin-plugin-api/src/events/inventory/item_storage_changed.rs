use super::super::FromIntoEvent;
use crate::wit::pumpkin::plugin::event::{Event, EventType, ItemStorageChangedEventData};

/// Event triggered once per tick when hoppers changed plugin item storages.
pub struct ItemStorageChangedEvent;
impl FromIntoEvent for ItemStorageChangedEvent {
    const EVENT_TYPE: EventType = EventType::ItemStorageChangedEvent;
    type Data = ItemStorageChangedEventData;

    fn data_from_event(event: Event) -> Self::Data {
        match event {
            Event::ItemStorageChangedEvent(data) => data,
            _ => panic!("unexpected event"),
        }
    }

    fn data_into_event(data: Self::Data) -> Event {
        Event::ItemStorageChangedEvent(data)
    }
}
