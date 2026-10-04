use super::super::FromIntoEvent;
use crate::wit::pumpkin::plugin::event::{Event, EventType, LivingDeathEventData};

/// A living entity took fatal damage (NeoForge `LivingDeathEvent`, Fabric `ServerLivingEntityEvents.ALLOW_DEATH`). Cancel and heal it to keep it alive.
pub struct LivingDeathEvent;
impl FromIntoEvent for LivingDeathEvent {
    const EVENT_TYPE: EventType = EventType::LivingDeathEvent;
    type Data = LivingDeathEventData;

    fn data_from_event(event: Event) -> Self::Data {
        match event {
            Event::LivingDeathEvent(data) => data,
            _ => panic!("unexpected event"),
        }
    }

    fn data_into_event(data: Self::Data) -> Event {
        Event::LivingDeathEvent(data)
    }
}
