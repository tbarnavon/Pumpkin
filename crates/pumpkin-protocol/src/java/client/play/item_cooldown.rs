use pumpkin_data::packet::clientbound::play::COOLDOWN;

use crate::codec::var_int::VarInt;
use pumpkin_macros::java_packet;

use crate::ClientPacket;
use crate::ser::NetworkWriteExt;
use pumpkin_util::version::JavaMinecraftVersion;

#[java_packet(COOLDOWN)]
pub struct CItemCooldown {
    pub group: String,
    pub cooldown: VarInt,
}

impl CItemCooldown {
    #[must_use]
    pub const fn new(group: String, cooldown: VarInt) -> Self {
        Self { group, cooldown }
    }
}

impl ClientPacket for CItemCooldown {
    fn write_packet_data(
        &self,
        mut write: impl std::io::Write,
        version: &JavaMinecraftVersion,
    ) -> Result<(), crate::ser::WritingError> {
        if *version >= JavaMinecraftVersion::V_1_21_2 {
            write.write_string(&self.group)?;
        } else {
            // Before 1.21.2 cooldowns are per item (`ClientboundCooldownPacket(Item, int)`), and the
            // group is the item's id.
            let item =
                pumpkin_data::item::Item::from_registry_key(&self.group).ok_or_else(|| {
                    crate::ser::WritingError::Message(format!(
                        "No item for cooldown {}",
                        self.group
                    ))
                })?;
            write.write_var_int(&VarInt(i32::from(item.id)))?;
        }
        write.write_var_int(&self.cooldown)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use pumpkin_util::version::JavaMinecraftVersion;

    use super::CItemCooldown;
    use crate::{ClientPacket, VarInt, ser::NetworkWriteExt};

    #[test]
    fn sends_the_item_id_for_1_21_1() {
        let packet = CItemCooldown::new("minecraft:ender_pearl".into(), VarInt(20));
        let mut bytes = Vec::new();
        packet
            .write_packet_data(&mut bytes, &JavaMinecraftVersion::V_1_21)
            .unwrap();
        let mut expected = Vec::new();
        expected
            .write_var_int(&VarInt(i32::from(pumpkin_data::item::Item::ENDER_PEARL.id)))
            .unwrap();
        expected.write_var_int(&VarInt(20)).unwrap();
        assert_eq!(bytes, expected);
    }
}
