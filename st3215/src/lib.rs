use embedded_io_async::Write;

pub struct InstructionPacket {
    id: ServoIdOrBroadcast,
    instruction: Instruction,
}

impl InstructionPacket {
    fn length() -> u8 {
        todo!()
    }
    fn checksum() -> u8 {
        todo!()
    }
    async fn send(stream: impl Write) -> Vec<u8> {
        todo!()
    }
}

pub struct ServoIdOrBroadcast(pub u8);

/** ID No. 254 is a broadcast ID */
#[repr(transparent)]
pub struct ServoId(u8);

impl ServoId {
    pub fn new(id: u8) -> Option<Self> {
        if id == 254 {
            None
        } else {
            Some(Self(id))
        }
    }
}

pub enum Instruction {
    Ping,
    // ...
}

pub struct ReplyPacket {
    id: ServoId,
    length: u8,
    current_state: CurrentState,
    parameters: [u8; 256],
}

#[repr(u8)]
pub enum CurrentState {
    Normal = 0,
}

#[cfg(test)]
mod tests {
    use super::*;

    // I expect that most people will create a bunch of static constants like this, so they will
    // never need to have an unwrap() that can panic at runtime. Let's see if that pans out.
    const TEST_SERVO_ID: ServoId = ServoId.new(1).unwrap();

    #[test]
    fn comparing_const_to_dynamic() {
        assert_eq!(TEST_SERVO_ID, ServoId.new(1).unwrap());
    }

}
