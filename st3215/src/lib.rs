#![allow(dead_code, unused_variables)]
use embedded_io_async::{Read, ReadExactError, Write};

pub struct InstructionPacket {
    id: ServoIdOrBroadcast,
    instruction: Instruction,
}

impl InstructionPacket {
    fn length(&self) -> u8 {
        2 + self.instruction.parameters().len() as u8
    }
    pub(crate) fn checksum(&self) -> u8 {
        let mut sum = self.id.0.wrapping_add(self.length()).wrapping_add(self.instruction.code());
        for byte in self.instruction.parameters() {
            sum = sum.wrapping_add(*byte);
        }
        !sum

    }
    pub async fn write<W: Write>(&self, mut stream: W) -> Result<(), W::Error> {
        stream.write_all(&[0xff, 0xff, self.id.0, self.length(), self.instruction.code()]).await?;
        // TODO: work out what happens if something else tries to write to the stream while we're paused here.
        // I guess this is why the "client owns the stream" model is so popular?
        stream.write_all(self.instruction.parameters()).await?;
        stream.write_all(&[self.checksum()]).await?;
        Ok(())
    }
}

pub struct ServoIdOrBroadcast(pub u8);

/** ID No. 254 is a broadcast ID */
#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ServoId(u8);

impl ServoId {
    pub const fn new(id: u8) -> Option<Self> {
        if id == 254 {
            None
        } else {
            Some(Self(id))
        }
    }
}

pub enum Instruction {
    /** Query the working status (0x01) */
    Ping,
    /** Query the Characters in the Control Table (0x02) */
    ReadData {parameters: [u8; 2]},
    /** Write characters into the control table (0x03) */
    WriteData {parameters: [u8; 256]}, // >= 1
    /** Similar to WRITE DATA, the control character does not act immediately after writing until the ACTION instruction arrives. (0x04) */
    RegWriteData {parameters: [u8; 256]}, // Not less than 2
    /** Actions that trigger REG WRITE writes (0x05) */
    Action,
    /** For simultaneous control of multiple servos (0x83) */
    SyncWrite {parameters: [u8; 256]}, // Not less than 2
    /** Reset control table to factory value (0x06) */
    Reset,
}
impl Instruction {
    fn code(&self) -> u8 {
        match self {
            Instruction::Ping => 0x01,
            Instruction::ReadData {..} => 0x02,
            Instruction::WriteData {..} => 0x03,
            Instruction::RegWriteData { .. } => 0x04,
            Instruction::Action => 0x05,
            Instruction::SyncWrite { .. } => 0x83,
            Instruction::Reset => 0x06,
        }
    }
    fn parameters(&self) -> &[u8] {
        match self {
            Instruction::Ping => &[],
            Instruction::ReadData {parameters} => parameters,
            Instruction::WriteData {parameters} => parameters,
            Instruction::RegWriteData {parameters} => parameters,
            Instruction::Action => &[],
            Instruction::SyncWrite {parameters} => parameters,
            Instruction::Reset => &[],
        }
    }
}

pub struct ReplyPacket {
    id: ServoId,
    length: u8,
    current_state: CurrentState,
    parameters: [u8; 256],
}

impl ReplyPacket {
    pub async fn read<R: Read>(mut stream: R) -> Result<Self,ReadExactError< R::Error>> {
        let mut buffer = [0u8; 5];
        stream.read_exact(&mut buffer)
            .await?;
        debug_assert!(buffer[0] == 0xff);
        debug_assert!(buffer[1] == 0xff);
        let id = ServoId::new(buffer[2]).unwrap();
        let length = buffer[3].saturating_sub(2);
        let current_state = buffer[4];

        let mut res = Self {
            id,
            length,
            current_state: CurrentState::Normal,
            // FIXME: refactor this to use maybeuninit or some smol vec impl for a tiny speedup?
            parameters: [0u8; 256],
        };
        stream.read_exact(&mut res.parameters[..length as usize])
            .await?;

        let mut checksum = [0u8; 1];
        stream.read_exact(&mut checksum)
            .await?;
        
        // FIXME: add an error variant for this instead of panicking
        assert_eq!(res.checksum(), checksum[0]);
        Ok(res)

    }
    fn checksum(&self) -> u8 {
        // FIXME: it might be better to dump everything off the wire into a buffer and checksum that,
        // rather than parsing and then partiallu un-parsing to checksum.
        let mut sum = self.id.0.wrapping_add(self.length.wrapping_add(2)).wrapping_add(self.current_state.as_u8());
        for byte in &self.parameters[..self.length as usize] {
            sum = sum.wrapping_add(*byte);
        }
        !sum
    }
    pub fn parameters(&self) -> &[u8] {
        &self.parameters[..self.length as usize]
    }
}

#[repr(u8)]
pub enum CurrentState {
    Normal = 0,
}
impl CurrentState {
    fn from_u8(value: u8) -> Option<Self> {
        match value {
            0 => Some(Self::Normal),
            _ => None,
        }
    }
    fn as_u8(&self) -> u8 {
        match self {
            Self::Normal => 0,    
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // I expect that most people will create a bunch of static constants like this, so they will
    // never need to have an unwrap() that can panic at runtime. Let's see if that pans out.
    const TEST_SERVO_ID: ServoId = ServoId::new(1).unwrap();

    #[test]
    fn comparing_const_to_dynamic() {
        assert_eq!(TEST_SERVO_ID, ServoId::new(1).unwrap());
    }

    /** example from `1.3.1 Query status instruction PING` */
    #[futures_test::test]
    async fn query_status_instruction_ping_1_3_1() {
        let packet = InstructionPacket {
            id: ServoIdOrBroadcast(1),
            instruction: Instruction::Ping,
        };
        let mut stream: Vec<u8> = Vec::new();
        assert_eq!(packet.length(), 0x02);
        assert_eq!(packet.checksum(), 0xfB);
        packet.write(&mut stream).await;
        assert_eq!(stream, vec![0xff, 0xff, 0x01, 0x02, 0x01, 0xfB]);
    }

    /** example from `1.3.2 READ DATA` */
    #[futures_test::test]
    async fn read_data_1_3_2_instruction() {
        let packet = InstructionPacket {
            id: ServoIdOrBroadcast(1),
            instruction: Instruction::ReadData {parameters: [0x38, 0x02]},
        };
        let mut stream: Vec<u8> = Vec::new();
        assert_eq!(packet.length(), 0x04);
        assert_eq!(packet.checksum(), 0xbe);
        packet.write(&mut stream).await;
        assert_eq!(stream, vec![0xff, 0xff, 0x01, 0x04, 0x02, 0x38, 0x02, 0xbe]);
    }

    /** example from `1.3.2 READ DATA` */
    #[futures_test::test]
    async fn read_data_1_3_2_response() {
        let received_data_frame: Vec<u8> = vec![0xff, 0xff, 0x01, 0x04, 0x00, 0x18, 0x05, 0xDD];
        let mut stream: &[u8] = &received_data_frame;
        let packet = ReplyPacket::read(&mut stream).await.unwrap();

        assert_eq!(packet.parameters(), &[0x18, 0x05]);
    }

}
