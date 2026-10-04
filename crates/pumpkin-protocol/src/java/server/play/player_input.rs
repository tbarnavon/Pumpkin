use crate::{
    ServerPacket,
    ser::{NetworkReadExt, ReadingError},
};
use pumpkin_data::packet::serverbound::play::PLAYER_INPUT;
use pumpkin_macros::java_packet;
use pumpkin_util::version::JavaMinecraftVersion;

#[java_packet(PLAYER_INPUT)]
pub struct SPlayerInput {
    // Yep, exactly how it looks like
    pub input: i8,
}

impl SPlayerInput {
    pub const FORWARD: i8 = 1;
    pub const BACKWARD: i8 = 2;
    pub const LEFT: i8 = 4;
    pub const RIGHT: i8 = 8;
    pub const JUMP: i8 = 16;
    pub const SNEAK: i8 = 32;
    pub const SPRINT: i8 = 64;
}

impl<'a> ServerPacket<'a> for SPlayerInput {
    fn read(bytebuf: &mut &'a [u8], version: &JavaMinecraftVersion) -> Result<Self, ReadingError> {
        if version >= &JavaMinecraftVersion::V_1_21_2 {
            Ok(Self {
                input: bytebuf.get_i8()?,
            })
        } else {
            let sideways = bytebuf.get_f32_be()?;
            let forward = bytebuf.get_f32_be()?;
            // Before 1.21.2: jumping is bit 1 and sneaking bit 2 of one flag byte.
            let flags = bytebuf.get_u8()?;
            let jumping = flags & 1 != 0;
            let sneaking = flags & 2 != 0;

            let mut input: i8 = 0;
            if forward > 0.0 {
                input |= Self::FORWARD;
            } else if forward < 0.0 {
                input |= Self::BACKWARD;
            }
            if sideways > 0.0 {
                input |= Self::LEFT;
            } else if sideways < 0.0 {
                input |= Self::RIGHT;
            }
            if jumping {
                input |= Self::JUMP;
            }
            if sneaking {
                input |= Self::SNEAK;
            }

            Ok(Self { input })
        }
    }
}

impl crate::ClientPacket for SPlayerInput {
    fn write_packet_data(
        &self,
        mut write: impl std::io::Write,
        version: &JavaMinecraftVersion,
    ) -> Result<(), crate::ser::WritingError> {
        use crate::ser::NetworkWriteExt;
        if version >= &JavaMinecraftVersion::V_1_21_2 {
            write.write_i8(self.input)?;
        } else {
            let mut sideways: f32 = 0.0;
            let mut forward: f32 = 0.0;
            if (self.input & Self::FORWARD) != 0 {
                forward += 1.0;
            }
            if (self.input & Self::BACKWARD) != 0 {
                forward -= 1.0;
            }
            if (self.input & Self::LEFT) != 0 {
                sideways += 1.0;
            }
            if (self.input & Self::RIGHT) != 0 {
                sideways -= 1.0;
            }
            let jumping = (self.input & Self::JUMP) != 0;
            let sneaking = (self.input & Self::SNEAK) != 0;
            write.write_f32_be(sideways)?;
            write.write_f32_be(forward)?;
            write.write_u8(u8::from(jumping) | (u8::from(sneaking) << 1))?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn player_input_1_21_reads_flag_byte() {
        // 1.21.1 sends sideways and forward as floats, then jumping (1) and sneaking (2) as one byte.
        let bytes = [0x3f, 0x80, 0, 0, 0xbf, 0x80, 0, 0, 0x03];
        let mut slice = bytes.as_slice();
        let packet = SPlayerInput::read(&mut slice, &JavaMinecraftVersion::V_1_21).unwrap();
        assert!(slice.is_empty());
        assert_eq!(
            packet.input,
            SPlayerInput::LEFT | SPlayerInput::BACKWARD | SPlayerInput::JUMP | SPlayerInput::SNEAK
        );
    }
}
