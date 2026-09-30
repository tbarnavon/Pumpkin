use super::super::FromIntoEvent;
use crate::wit::pumpkin::plugin::event::{Event, EventType, PlayerSleepCheckEventData};

/// A player tries to sleep; set `time-ok` or `monsters-ok` to allow or forbid it (Fabric `EntitySleepEvents.ALLOW_SLEEP_TIME` and `ALLOW_NEARBY_MONSTERS`).
pub struct PlayerSleepCheckEvent;
impl FromIntoEvent for PlayerSleepCheckEvent {
    const EVENT_TYPE: EventType = EventType::PlayerSleepCheckEvent;
    type Data = PlayerSleepCheckEventData;

    fn data_from_event(event: Event) -> Self::Data {
        match event {
            Event::PlayerSleepCheckEvent(data) => data,
            _ => panic!("unexpected event"),
        }
    }

    fn data_into_event(data: Self::Data) -> Event {
        Event::PlayerSleepCheckEvent(data)
    }
}
