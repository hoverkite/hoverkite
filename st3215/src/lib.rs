#![allow(dead_code, unused_variables)]
use embedded_io_async::{Read, ReadExactError, Write};
use tinyvec::ArrayVec;

pub struct InstructionPacket {
    id: ServoIdOrBroadcast,
    instruction: Instruction,
}

impl InstructionPacket {
    fn effective_data_length(&self) -> u8 {
        2 + self.instruction.parameters_len()
    }
    pub fn write_to_buf(
        &self,
        // FIXME: how to make capacity a const generic parameter?
        // When I try, it tells me that buf.len() doesn't exist as a method.
        buf: &mut ArrayVec<[u8; 256]>,
    ) {
        let buf_start_len = buf.len();
        // FIXME: these currently panic if the buffer is full. I should probably return an error instead.
        buf.extend_from_slice(&[
            0xff,
            0xff,
            self.id.0,
            self.effective_data_length(),
            // TODO: consider making the instruction write its code to buf instead of writing code
            // and parameters separately.
            self.instruction.code(),
        ]);

        self.instruction.write_parameters_to_buf(buf);

        let checksum = !buf[(buf_start_len + 2)..]
            .iter()
            .fold(0u8, |acc, &byte| acc.wrapping_add(byte));
        buf.push(checksum);
    }

    pub async fn write<W: Write>(&self, mut stream: W) -> Result<(), W::Error> {
        let mut buf = ArrayVec::new();
        self.write_to_buf(&mut buf);

        stream.write_all(&buf).await?;

        Ok(())
    }
}

pub struct ServoIdOrBroadcast(pub u8);

impl ServoIdOrBroadcast {
    const BROADCAST: Self = Self(254);
}

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
    ReadData { parameters: [u8; 2] },
    /** Write characters into the control table (0x03) */
    WriteData {
        head_address: u8,
        data: ArrayVec<[u8; 256]>,
    }, // >= 1
    /** Similar to WRITE DATA, the control character does not act immediately after writing until the ACTION instruction arrives. (0x04) */
    RegWriteData { parameters: ArrayVec<[u8; 256]> }, // Not less than 2
    /** Actions that trigger REG WRITE writes (0x05) */
    Action,
    /** For simultaneous control of multiple servos (0x83) */
    SyncWrite { parameters: ArrayVec<[u8; 256]> }, // Not less than 2
    /** Reset control table to factory value (0x06) */
    Reset,
}
impl Instruction {
    fn code(&self) -> u8 {
        match self {
            Instruction::Ping => 0x01,
            Instruction::ReadData { .. } => 0x02,
            Instruction::WriteData { .. } => 0x03,
            Instruction::RegWriteData { .. } => 0x04,
            Instruction::Action => 0x05,
            Instruction::SyncWrite { .. } => 0x83,
            Instruction::Reset => 0x06,
        }
    }
    /** Convenance wrapper for testing. You probably want to use write_parameters_to_buf() instead. */
    fn parameters_as_buf(&self) -> ArrayVec<[u8; 256]> {
        let mut buf = ArrayVec::new();
        self.write_parameters_to_buf(&mut buf);
        buf
    }
    fn write_parameters_to_buf(&self, buf: &mut ArrayVec<[u8; 256]>) {
        match self {
            Instruction::Ping => {}
            Instruction::ReadData { parameters } => buf.extend_from_slice(parameters),
            Instruction::WriteData {
                head_address,
                data: values,
            } => {
                buf.push(*head_address);
                buf.extend_from_slice(values)
            }
            Instruction::RegWriteData { parameters } => buf.extend_from_slice(parameters),
            Instruction::Action => {}
            Instruction::SyncWrite { parameters } => buf.extend_from_slice(parameters),
            Instruction::Reset => {}
        }
    }

    // TODO: this is a bit non-dry. Write a proptest that makes sure this matches parameters_as_buf().len()
    fn parameters_len(&self) -> u8 {
        match self {
            Instruction::Ping => 0,
            Instruction::ReadData { parameters } => parameters.len() as u8,
            Instruction::WriteData {
                head_address,
                data: values,
            } => values.len() as u8 + 1,
            Instruction::RegWriteData { parameters } => parameters.len() as u8,
            Instruction::Action => 0,
            Instruction::SyncWrite { parameters } => parameters.len() as u8,
            Instruction::Reset => 0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ReplyPacket {
    id: ServoId,
    current_state: CurrentState,
    parameters: ArrayVec<[u8; 256]>,
}

impl ReplyPacket {
    pub async fn read<R: Read>(mut stream: R) -> Result<Self, ReadExactError<R::Error>> {
        let mut buffer = [0u8; 5];
        stream.read_exact(&mut buffer).await?;
        debug_assert!(buffer[0] == 0xff);
        debug_assert!(buffer[1] == 0xff);
        let id = ServoId::new(buffer[2]).unwrap();
        let length = buffer[3].saturating_sub(2);
        let current_state = buffer[4];

        let mut res = Self {
            id,
            current_state: CurrentState::Normal,
            // FIXME: refactor this to use maybeuninit or some smol vec impl for a tiny speedup?
            parameters: ArrayVec::new(),
        };
        res.parameters.resize(length as usize, 0);
        stream.read_exact(&mut res.parameters[..]).await?;

        let mut checksum = [0u8; 1];
        stream.read_exact(&mut checksum).await?;

        // FIXME: add an error variant for this instead of panicking
        assert_eq!(res.checksum(), checksum[0]);
        Ok(res)
    }
    fn checksum(&self) -> u8 {
        // FIXME: it might be better to dump everything off the wire into a buffer and checksum that,
        // rather than parsing and then partiallu un-parsing to checksum.
        let mut sum = self
            .id
            .0
            .wrapping_add((self.parameters().len() as u8).wrapping_add(2))
            .wrapping_add(self.current_state.as_u8());
        for byte in &self.parameters {
            sum = sum.wrapping_add(*byte);
        }
        !sum
    }
    pub fn parameters(&self) -> &[u8] {
        &self.parameters
    }
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
    use tinyvec::array_vec;

    // I expect that most people will create a bunch of static constants like this, so they will
    // never need to have an unwrap() that can panic at runtime. Let's see if that pans out.
    const TEST_SERVO_ID: ServoId = ServoId::new(1).unwrap();

    #[test]
    fn comparing_const_to_dynamic() {
        assert_eq!(TEST_SERVO_ID, ServoId::new(1).unwrap());
    }

    /**
     * Example 1 reads the working state of the steering gear with ID number 1.
     * (example from `1.3.1 Query status instruction PING`)
     * */
    #[futures_test::test]
    async fn example_1_query_status_instruction_ping_1_3_1() {
        let packet = InstructionPacket {
            id: ServoIdOrBroadcast(1),
            instruction: Instruction::Ping,
        };
        let mut stream: Vec<u8> = Vec::new();
        assert_eq!(packet.effective_data_length(), 0x02);
        assert_eq!(packet.instruction.parameters_as_buf().as_slice(), &[]);
        packet.write(&mut stream).await.unwrap();
        assert_eq!(stream, vec![0xff, 0xff, 0x01, 0x02, 0x01, 0xfB]);
    }

    /**
     * Example 2 Read the current position of the servo with ID 1
     * (example from `1.3.2 READ DATA`, first part)
     */
    #[futures_test::test]
    async fn example_2_read_data_instruction() {
        let packet = InstructionPacket {
            id: ServoIdOrBroadcast(1),
            instruction: Instruction::ReadData {
                parameters: [0x38, 0x02],
            },
        };
        let mut stream: Vec<u8> = Vec::new();
        assert_eq!(packet.effective_data_length(), 0x04);
        assert_eq!(
            packet.instruction.parameters_as_buf().as_slice(),
            &[0x38, 0x02]
        );
        packet.write(&mut stream).await.unwrap();
        assert_eq!(stream, vec![0xff, 0xff, 0x01, 0x04, 0x02, 0x38, 0x02, 0xbe]);
    }

    /**
     * Example 2 Read the current position of the servo with ID 1
     * (example from `1.3.2 READ DATA`, second part)
     */
    #[futures_test::test]
    async fn example_2_read_data_response() {
        let received_data_frame: Vec<u8> = vec![0xff, 0xff, 0x01, 0x04, 0x00, 0x18, 0x05, 0xDD];
        let mut stream: &[u8] = &received_data_frame;
        let packet = ReplyPacket::read(&mut stream).await.unwrap();

        assert_eq!(
            packet,
            ReplyPacket {
                id: ServoId::new(1).unwrap(),
                current_state: CurrentState::Normal,
                parameters: array_vec![0x18, 0x05],
            }
        );
    }

    /**
     * Example 3 sets an ID of any number to 1
     * (example from `1.3.3 WRITE DATA`)
     */
    #[futures_test::test]
    async fn example_3_broadcast_set_id() {
        let packet = InstructionPacket {
            id: ServoIdOrBroadcast::BROADCAST,
            instruction: Instruction::WriteData {
                // FIXME: split this into "head address" and array of values
                head_address: 0x05,
                data: array_vec!(0x01),
            },
        };
        let mut stream: Vec<u8> = Vec::new();
        assert_eq!(packet.effective_data_length(), 0x04);
        assert_eq!(
            packet.instruction.parameters_as_buf().as_slice(),
            [0x05, 0x01]
        );
        packet.write(&mut stream).await.unwrap();
        assert_eq!(stream, vec![0xff, 0xff, 0xfe, 0x04, 0x03, 0x05, 0x01, 0xf4]);
    }

    /**
     * Example 4 controls the ID1 servo to rotate to 2048 at a speed of 1000 seconds.
     * (first part)
     */
    #[futures_test::test]
    async fn example_4_control_servo_instruction() {
        let packet = InstructionPacket {
            id: ServoIdOrBroadcast(1),
            instruction: Instruction::WriteData {
                head_address: 0x2a,
                data: array_vec!(0x00, 0x08, 0x00, 0x00, 0xe8, 0x03),
            },
        };
        let mut stream: Vec<u8> = Vec::new();
        assert_eq!(packet.effective_data_length(), 0x09);
        assert_eq!(
            packet.instruction.parameters_as_buf().as_slice(),
            [0x2au8, 0x00, 0x08, 0x00, 0x00, 0xe8, 0x03]
        );
        packet.write(&mut stream).await.unwrap();
        assert_eq!(
            stream,
            vec![0xff, 0xff, 0x01, 0x09, 0x03, 0x2a, 0x00, 0x08, 0x00, 0x00, 0xe8, 0x03, 0xd5]
        );
    }

    /**
     * Example 4 controls the ID1 servo to rotate to 2048 at a speed of 1000 seconds.
     * (second part)
     */
    #[futures_test::test]
    async fn example_4_control_servo_response() {
        let received_data_frame: Vec<u8> = vec![0xff, 0xff, 0x01, 0x02, 0x00, 0xFC];
        let mut stream: &[u8] = &received_data_frame;
        let packet = ReplyPacket::read(&mut stream).await.unwrap();

        assert_eq!(
            packet,
            ReplyPacket {
                id: ServoId::new(1).unwrap(),
                current_state: CurrentState::Normal,
                parameters: array_vec![],
            }
        );
    }
}
