#![cfg_attr(not(feature = "std"), no_std)]

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
    Recenter,
    BatteryReport,
    Relax,
    PowerOff,
}

#[cfg(feature = "std")]
impl Command {
    pub fn write_to(&self, mut writer: impl std::io::Write) -> std::io::Result<()> {
        match self {
            Command::SetMaxSpeed(max_speed) => {
                writer.write_all(&[b'S'])?;
                writer.write_all(&max_speed.start().to_le_bytes())?;
                writer.write_all(&max_speed.end().to_le_bytes())?;
            }
            Command::SetSpringConstant(spring_constant) => {
                writer.write_all(&[b'K'])?;
                writer.write_all(&spring_constant.to_le_bytes())?;
            }
            Command::SetTarget(target) => {
                writer.write_all(&[b'T'])?;
                writer.write_all(&target.to_le_bytes())?;
            }
            Command::Recenter => writer.write_all(&[b'e'])?,
            Command::BatteryReport => writer.write_all(&[b'b'])?,
            Command::Relax => writer.write_all(&[b'n'])?,
            Command::PowerOff => writer.write_all(&[b'p'])?,
        };
        Ok(())
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

#[cfg(feature = "std")]
impl DirectedCommand {
    pub fn write_to(&self, mut writer: impl std::io::Write) -> std::io::Result<()> {
        match self {
            DirectedCommand::Left(command) => command.write_to(writer)?,
            DirectedCommand::Right(command) => {
                let mut encoded: std::vec::Vec<u8> = vec![];
                command.write_to(&mut encoded)?;
                writer.write_all(&[b'F', encoded.len() as u8])?;
                writer.write_all(&encoded)?;
            }
        }
        Ok(())
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

#[cfg(feature = "std")]
#[cfg(test)]
mod tests {
    use super::*;

    mod basic {
        use super::*;

        #[test]
        fn power_off() {
            let command = Command::PowerOff;
            let mut buf = vec![];
            command.write_to(&mut buf).unwrap();
            assert_eq!(buf, [b'p']);
        }

        #[test]
        fn set_target() {
            let command = Command::SetTarget(42);
            let mut buf = vec![];
            command.write_to(&mut buf).unwrap();
            assert_eq!(buf, [b'T', 42, 0, 0, 0, 0, 0, 0, 0]);
        }
    }

    mod directed {
        use super::*;

        #[test]
        fn power_off_left() {
            let command = DirectedCommand::Left(Command::PowerOff);
            let mut buf = vec![];
            command.write_to(&mut buf).unwrap();
            assert_eq!(buf, [b'p']);
        }

        #[test]
        fn power_off_right() {
            let command = DirectedCommand::Right(Command::PowerOff);
            let mut buf = vec![];
            command.write_to(&mut buf).unwrap();
            assert_eq!(buf, [b'F', 1, b'p']);
        }
    }
}
