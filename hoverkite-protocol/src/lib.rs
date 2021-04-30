#![cfg_attr(not(feature = "std"), no_std)]

use core::ops::RangeInclusive;

/// A compatibility shim that unifies std::io::Write and embedded_hal::blocking::serial::Write
/// TODO: propose the following impl to embedded_hal crate:
///
/// #[cfg(feature = "std")]
/// impl<T: std::io::Write> embedded_hal::blocking::serial::Write<u8> for T {...}
#[cfg(feature = "std")]
pub struct WriteCompat<T: std::io::Write>(pub T);

#[cfg(feature = "std")]
impl<W: std::io::Write> embedded_hal::blocking::serial::Write<u8> for WriteCompat<W> {
    type Error = std::io::Error;

    fn bwrite_all(&mut self, buffer: &[u8]) -> Result<(), Self::Error> {
        self.0.write_all(buffer)
    }

    fn bflush(&mut self) -> Result<(), Self::Error> {
        self.0.flush()
    }
}

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
    ReportBattery,
    RemoveTarget,
    PowerOff,
}

impl Command {
    // FIXME: this goes away once the blanket impl exists.
    #[cfg(feature = "std")]
    pub fn write_to_std(&self, writer: impl std::io::Write) -> std::io::Result<()> {
        self.write_to(WriteCompat(writer))
    }

    pub fn write_to<W>(&self, mut writer: W) -> Result<(), W::Error>
    where
        W: embedded_hal::blocking::serial::Write<u8>,
    {
        match self {
            Command::SetMaxSpeed(max_speed) => {
                writer.bwrite_all(&[b'S'])?;
                writer.bwrite_all(&max_speed.start().to_le_bytes())?;
                writer.bwrite_all(&max_speed.end().to_le_bytes())?;
            }
            Command::SetSpringConstant(spring_constant) => {
                writer.bwrite_all(&[b'K'])?;
                writer.bwrite_all(&spring_constant.to_le_bytes())?;
            }
            Command::SetTarget(target) => {
                writer.bwrite_all(&[b'T'])?;
                writer.bwrite_all(&target.to_le_bytes())?;
            }
            Command::Recenter => writer.bwrite_all(&[b'e'])?,
            Command::ReportBattery => writer.bwrite_all(&[b'b'])?,
            Command::RemoveTarget => writer.bwrite_all(&[b'n'])?,
            Command::PowerOff => writer.bwrite_all(&[b'p'])?,
        };
        Ok(())
    }
}

pub struct SecondaryCommand(pub Command);

// I'm not expecting to need this anywhere other than on the host.
// Famous last words.
#[cfg(feature = "std")]
impl SecondaryCommand {
    pub fn write_to_std(&self, mut writer: impl std::io::Write) -> std::io::Result<()> {
        let mut encoded: std::vec::Vec<u8> = vec![];
        self.0.write_to_std(&mut encoded)?;
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
            command.write_to_std(&mut buf).unwrap();
            assert_eq!(buf, [b'p']);
        }

        #[test]
        fn set_target() {
            let command = Command::SetTarget(42);
            let mut buf = vec![];
            command.write_to_std(&mut buf).unwrap();
            assert_eq!(buf, [b'T', 42, 0, 0, 0, 0, 0, 0, 0]);
        }
    }

    mod secondary_command {
        use super::*;

        #[test]
        fn power_off_secondary() {
            let command = SecondaryCommand(Command::PowerOff);
            let mut buf = vec![];
            command.write_to_std(&mut buf).unwrap();
            assert_eq!(buf, [b'F', 1, b'p']);
        }
    }
}
