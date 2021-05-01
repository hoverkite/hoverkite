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

pub struct SecondaryCommand(pub Command);

#[cfg(feature = "std")]
impl SecondaryCommand {
    pub fn write_to(&self, mut writer: impl std::io::Write) -> std::io::Result<()> {
        let mut encoded: std::vec::Vec<u8> = vec![];
        self.0.write_to(&mut encoded)?;
        writer.write_all(&[b'F', encoded.len() as u8])?;
        writer.write_all(&encoded)?;
        Ok(())
    }
}

// ... and then we could have an enum that represents all possible messages
// that can be sent/received over the wire that looks something like this?
// enum Message {
//     Command(Command),
//     SecondaryCommand(SecondaryCommand),
//     LogMessage(LogMessage),
//     SecondaryLogMessage(SecondaryLogMessage),
//     CurrentPosition(CurrentPosition)
//     SecondaryCurrentPosition(SecondaryCurrentPosition)
// }

#[cfg(feature = "std")]
#[cfg(test)]
mod tests {
    use super::*;

    mod command {
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

    mod secondary_command {
        use super::*;

        #[test]
        fn power_off_secondary() {
            let command = SecondaryCommand(Command::PowerOff);
            let mut buf = vec![];
            command.write_to(&mut buf).unwrap();
            assert_eq!(buf, [b'F', 1, b'p']);
        }
    }
}
