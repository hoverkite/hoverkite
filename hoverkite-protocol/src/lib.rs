use core::ops::RangeInclusive;

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Side {
    Left,
    Right,
}

impl Side {
    pub fn opposite(self) -> Self {
        match self {
            Self::Left => Self::Right,
            Self::Right => Self::Left,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Command {
    // FIXME: stop using RangeInclusive, so we can derive Copy
    SetMaxSpeed(RangeInclusive<i16>),
    SetSpringConstant(u16),
    SetTarget(i64),
    Recenter,      // vec![b'e'] -
    BatteryReport, // vec![b'b'] -
    Relax,         // vec![b'n'] -
    PowerOff,      // vec![b'p'] -
}

impl From<Command> for Vec<u8> {
    fn from(command: Command) -> Vec<u8> {
        match command {
            Command::SetMaxSpeed(max_speed) => {
                let mut encoded = vec![b'S'];
                encoded.extend_from_slice(&max_speed.start().to_le_bytes());
                encoded.extend_from_slice(&max_speed.end().to_le_bytes());
                encoded
            }
            Command::SetSpringConstant(spring_constant) => {
                let mut encoded = vec![b'K'];
                encoded.extend_from_slice(&spring_constant.to_le_bytes());
                encoded
            }
            Command::SetTarget(target) => {
                let mut encoded = vec![b'T'];
                encoded.extend_from_slice(&target.to_le_bytes());
                encoded
            }
            Command::Recenter => vec![b'e'],
            Command::BatteryReport => vec![b'b'],
            Command::Relax => vec![b'n'],
            Command::PowerOff => vec![b'p'],
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

impl From<&DirectedCommand> for Side {
    fn from(command: &DirectedCommand) -> Side {
        match command {
            DirectedCommand::Left(_) => Side::Left,
            DirectedCommand::Right(_) => Side::Right,
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
