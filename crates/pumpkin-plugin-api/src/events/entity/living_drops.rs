use super::super::FromIntoEvent;
use crate::wit::pumpkin::plugin::event::{Event, EventType, LivingDropsEventData};

/// What a dying living entity drops (NeoForge `LivingDropsEvent`, `LivingExperienceDropEvent`). Replace `drops` or change `experience`; cancel to drop no items.
pub struct LivingDropsEvent;
impl FromIntoEvent for LivingDropsEvent {
    const EVENT_TYPE: EventType = EventType::LivingDropsEvent;
    type Data = LivingDropsEventData;

    fn data_from_event(event: Event) -> Self::Data {
        match event {
            Event::LivingDropsEvent(data) => data,
            _ => panic!("unexpected event"),
        }
    }

    fn data_into_event(data: Self::Data) -> Event {
        Event::LivingDropsEvent(data)
    }
}
