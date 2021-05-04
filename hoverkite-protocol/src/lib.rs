#![cfg_attr(not(feature = "std"), no_std)]

use core::{convert::TryInto, ops::RangeInclusive};

use nb::Error::{Other, WouldBlock};

/// A compatibility shim that unifies std::io::Write and embedded_hal::blocking::serial::Write
// TODO: propose the following impl to embedded_hal crate:
//
// #[cfg(feature = "std")]
// impl<W: std::io::Write> embedded_hal::blocking::serial::Write<u8> for W {...}
#[cfg(feature = "std")]
pub struct WriteCompat<W: std::io::Write>(pub W);

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
    ReportCharger,
    RemoveTarget,
    PowerOff,
}

impl Command {
    // FIXME: this goes away once the blanket impl exists.
    #[cfg(feature = "std")]
    pub fn write_to_std(&self, writer: impl std::io::Write) -> std::io::Result<()> {
        self.write_to(&mut WriteCompat(writer))
    }

    pub fn write_to<W>(&self, writer: &mut W) -> Result<(), W::Error>
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
            Command::ReportCharger => writer.bwrite_all(&[b'c'])?,
            Command::RemoveTarget => writer.bwrite_all(&[b'n'])?,
            Command::PowerOff => writer.bwrite_all(&[b'p'])?,
        };
        Ok(())
    }

    pub fn parse(buf: &[u8]) -> nb::Result<Self, ParseError> {
        let first = buf.get(0).ok_or(WouldBlock)?;
        match first {
            b'b' => Ok(Self::ReportBattery),
            b'c' => Ok(Self::ReportCharger),
            b'S' => {
                if buf.len() < 5 {
                    return Err(WouldBlock);
                }
                let min_power = i16::from_le_bytes(buf[1..3].try_into().unwrap());
                let max_power = i16::from_le_bytes(buf[3..5].try_into().unwrap());

                Ok(Self::SetMaxSpeed(min_power..=max_power))
            }
            b'K' => {
                if buf.len() < 3 {
                    return Err(WouldBlock);
                }
                let spring = u16::from_le_bytes(buf[1..3].try_into().unwrap()).into();
                Ok(Self::SetSpringConstant(spring))
            }
            b'n' => Ok(Self::RemoveTarget),
            b'T' => {
                if buf.len() < 9 {
                    return Err(WouldBlock);
                }
                let target = i64::from_le_bytes(buf[1..9].try_into().unwrap());
                Ok(Self::SetTarget(target))
            }
            b'e' => Ok(Self::Recenter),
            b'p' => Ok(Self::PowerOff),
            _ => Err(Other(ParseError)),
        }
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct ParseError;

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

pub enum Message {
    Command(Command),
    SecondaryCommand(SecondaryCommand),
    // TODO: add variants for the other things, or create a second enum that contains them.
    // Something like:
    //     LogMessage(LogMessage),
    //     SecondaryLogMessage(SecondaryLogMessage),
    //     CurrentPosition(CurrentPosition),
    //     SecondaryCurrentPosition(SecondaryCurrentPosition),
}

impl Message {
    pub fn parse(buf: &[u8]) -> nb::Result<Self, ParseError> {
        let first = buf.get(0).ok_or(WouldBlock)?;
        match first {
            b'F' => {
                let forward_length = buf.get(1).ok_or(WouldBlock)?;
                if buf.len() < *forward_length as usize + 2 {
                    return Err(WouldBlock);
                }
                Ok(Self::SecondaryCommand(SecondaryCommand(Command::parse(
                    &buf[2..],
                )?)))
            }
            _ => Ok(Self::Command(Command::parse(buf)?)),
        }
    }
}

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

    // TODO: see if it's possible to verify this round-trip property
    // using cargo-verify.
    mod round_trip {
        use super::*;

        fn assert_round_trip_equality(command: Command) {
            let mut buf = vec![];
            command.write_to_std(&mut buf).unwrap();
            let round_tripped_command = Command::parse(&buf).unwrap();

            assert_eq!(round_tripped_command, command)
        }

        #[test]
        fn power_off() {
            assert_round_trip_equality(Command::PowerOff);
        }

        #[test]
        fn set_target() {
            assert_round_trip_equality(Command::SetTarget(42));
        }
    }
}
