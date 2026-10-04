use pumpkin_data::packet::clientbound::play::ADD_EXPERIENCE_ORB;
use pumpkin_macros::java_packet;
use pumpkin_util::{math::vector3::Vector3, version::JavaMinecraftVersion};

use crate::{ClientPacket, VarInt, ser::NetworkWriteExt};

/// Spawns an experience orb. Before 1.21.5 orbs have their own spawn packet, which carries
/// the orb's value (`ClientboundAddExperienceOrbPacket`).
#[java_packet(ADD_EXPERIENCE_ORB)]
pub struct CAddExperienceOrb {
    pub entity_id: VarInt,
    pub position: Vector3<f64>,
    pub value: i16,
}

impl CAddExperienceOrb {
    #[must_use]
    pub const fn new(entity_id: VarInt, position: Vector3<f64>, value: i16) -> Self {
        Self {
            entity_id,
            position,
            value,
        }
    }
}

impl ClientPacket for CAddExperienceOrb {
    fn write_packet_data(
        &self,
        mut write: impl std::io::Write,
        _version: &JavaMinecraftVersion,
    ) -> Result<(), crate::ser::WritingError> {
        write.write_var_int(&self.entity_id)?;
        write.write_f64_be(self.position.x)?;
        write.write_f64_be(self.position.y)?;
        write.write_f64_be(self.position.z)?;
        write.write_i16_be(self.value)
    }
}

#[cfg(test)]
mod tests {
    use pumpkin_util::{math::vector3::Vector3, version::JavaMinecraftVersion};

    use super::CAddExperienceOrb;
    use crate::{ClientPacket, VarInt};

    #[test]
    fn writes_1_21_1_layout() {
        let packet = CAddExperienceOrb::new(VarInt(5), Vector3::new(1.0, 2.0, 3.0), 7);
        let mut bytes = Vec::new();
        packet
            .write_packet_data(&mut bytes, &JavaMinecraftVersion::V_1_21)
            .unwrap();
        let mut expected = vec![5];
        for v in [1.0f64, 2.0, 3.0] {
            expected.extend_from_slice(&v.to_be_bytes());
        }
        expected.extend_from_slice(&7i16.to_be_bytes());
        assert_eq!(bytes, expected);
    }
}
