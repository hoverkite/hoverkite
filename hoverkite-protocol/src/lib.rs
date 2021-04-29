use core::ops::RangeInclusive;

pub enum Command {
    SetMaxSpeed { max_speed: RangeInclusive<i16> },
    SetSpringConstant { spring_constant: u16 },
    SetTarget { target: i64 },
}

impl From<Command> for Vec<u8> {
    fn from(command: Command) -> Vec<u8> {
        match command {
            Command::SetMaxSpeed { max_speed } => {
                let mut encoded = vec![b'S'];
                encoded.extend_from_slice(&max_speed.start().to_le_bytes());
                encoded.extend_from_slice(&max_speed.end().to_le_bytes());
                encoded
            }
            Command::SetSpringConstant { spring_constant } => {
                let mut encoded = vec![b'K'];
                encoded.extend_from_slice(&spring_constant.to_le_bytes());
                encoded
            }
            Command::SetTarget { target } => {
                let mut encoded = vec![b'T'];
                encoded.extend_from_slice(&target.to_le_bytes());
                encoded
            }
        }
    }
}

pub enum DirectedCommand {
    // This is sent as-is.
    Right(Command),
    // Tell the right side to forward the command to the left side.
    // let mut wrapped_command = vec![b'F', command.len() as u8];
    // wrapped_command.extend_from_slice(command);
    // self.send_command(Side::Right, &wrapped_command)
    Left(Command),
    // ??? Should we add `Both(Command)`, or add command-specific forwarding
    // ??? logic to the firmware for SetMaxSpeed and SetSpringConstant?
}

impl From<DirectedCommand> for Vec<u8> {
    fn from(command: DirectedCommand) -> Vec<u8> {
        match command {
            DirectedCommand::Left(command) => command.into(),
            DirectedCommand::Right(command) => {
                let encoded: Vec<u8> = command.into();
                let mut wrapped_command = vec![b'F', encoded.len() as u8];
                wrapped_command.extend_from_slice(&encoded);
                wrapped_command
            }
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
