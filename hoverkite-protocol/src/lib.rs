#![cfg_attr(not(feature = "std"), no_std)]

use core::mem::size_of;
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
    SetSideLed(bool),
    SetOrangeLed(bool),
    SetRedLed(bool),
    SetGreenLed(bool),
    ReportBattery,
    ReportCharger,
    // FIXME: stop using RangeInclusive, so we can derive Copy
    SetMaxSpeed(RangeInclusive<i16>),
    SetSpringConstant(u16),
    SetTarget(i64),
    RemoveTarget,
    Recenter,
    IncrementTarget,
    DecrementTarget,
    PowerOff,
}

fn ascii_to_bool(char: u8) -> Result<bool, ParseError> {
    match char {
        b'1' => Ok(true),
        b'0' => Ok(false),
        _ => Err(ParseError),
    }
}

fn bool_to_ascii(on: bool) -> u8 {
    if on {
        b'1'
    } else {
        b'0'
    }
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
            Self::SetSideLed(on) => writer.bwrite_all(&[b'l', bool_to_ascii(*on)])?,
            Self::SetOrangeLed(on) => writer.bwrite_all(&[b'o', bool_to_ascii(*on)])?,
            Self::SetRedLed(on) => writer.bwrite_all(&[b'r', bool_to_ascii(*on)])?,
            Self::SetGreenLed(on) => writer.bwrite_all(&[b'g', bool_to_ascii(*on)])?,
            Self::SetMaxSpeed(max_speed) => {
                writer.bwrite_all(&[b'S'])?;
                writer.bwrite_all(&max_speed.start().to_le_bytes())?;
                writer.bwrite_all(&max_speed.end().to_le_bytes())?;
            }
            Self::SetSpringConstant(spring_constant) => {
                writer.bwrite_all(&[b'K'])?;
                writer.bwrite_all(&spring_constant.to_le_bytes())?;
            }
            Self::SetTarget(target) => {
                writer.bwrite_all(&[b'T'])?;
                writer.bwrite_all(&target.to_le_bytes())?;
            }
            Self::Recenter => writer.bwrite_all(&[b'e'])?,
            Self::ReportBattery => writer.bwrite_all(&[b'b'])?,
            Self::ReportCharger => writer.bwrite_all(&[b'c'])?,
            Self::RemoveTarget => writer.bwrite_all(&[b'n'])?,
            Self::IncrementTarget => writer.bwrite_all(&[b'+'])?,
            Self::DecrementTarget => writer.bwrite_all(&[b'-'])?,
            Self::PowerOff => writer.bwrite_all(&[b'p'])?,
        };
        Ok(())
    }

    pub fn parse(buf: &[u8]) -> nb::Result<Self, ParseError> {
        let command = match *buf {
            [] | [b'l'] | [b'o'] | [b'r'] | [b'g'] => return Err(WouldBlock),
            [b'l', on] => Self::SetSideLed(ascii_to_bool(on)?),
            [b'o', on] => Self::SetOrangeLed(ascii_to_bool(on)?),
            [b'r', on] => Self::SetRedLed(ascii_to_bool(on)?),
            [b'g', on] => Self::SetGreenLed(ascii_to_bool(on)?),
            [b'b'] => Self::ReportBattery,
            [b'c'] => Self::ReportCharger,
            [b'S', ref rest @ ..] => {
                if rest.len() < 4 {
                    return Err(WouldBlock);
                }
                if rest.len() > 4 {
                    return Err(Other(ParseError));
                }
                let min_power = i16::from_le_bytes(rest[..2].try_into().unwrap());
                let max_power = i16::from_le_bytes(rest[2..4].try_into().unwrap());
                Self::SetMaxSpeed(min_power..=max_power)
            }
            [b'K', ref rest @ ..] => {
                if rest.len() < size_of::<u16>() {
                    return Err(WouldBlock);
                }
                let bytes = rest.try_into().map_err(|_| Other(ParseError))?;
                let spring = u16::from_le_bytes(bytes);
                Self::SetSpringConstant(spring)
            }
            [b'n'] => Self::RemoveTarget,
            [b'T', ref rest @ ..] => {
                if rest.len() < size_of::<i64>() {
                    return Err(WouldBlock);
                }
                let bytes = rest.try_into().map_err(|_| Other(ParseError))?;
                let target = i64::from_le_bytes(bytes);
                Self::SetTarget(target)
            }
            [b'e'] => Self::Recenter,
            [b'+'] => Self::IncrementTarget,
            [b'-'] => Self::DecrementTarget,
            [b'p'] => Self::PowerOff,
            _ => return Err(Other(ParseError)),
        };
        Ok(command)
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct ParseError;

#[derive(Clone, Debug, Eq, PartialEq)]
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

#[derive(Clone, Debug, Eq, PartialEq)]
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

#[cfg(feature = "std")]
impl Message {
    pub fn write_to_std(&self, writer: impl std::io::Write) -> std::io::Result<()> {
        match self {
            Self::Command(c) => c.write_to_std(writer),
            Self::SecondaryCommand(sc) => sc.write_to_std(writer),
        }
    }
}

impl From<SecondaryCommand> for Message {
    fn from(val: SecondaryCommand) -> Self {
        Message::SecondaryCommand(val)
    }
}

impl From<Command> for Message {
    fn from(val: Command) -> Self {
        Message::Command(val)
    }
}

impl Message {
    pub fn parse(buf: &[u8]) -> nb::Result<Self, ParseError> {
        match *buf {
            [b'F', ref rest @ ..] => {
                if let [forward_length, ref rest @ ..] = *rest {
                    let forward_length = forward_length as usize;
                    if rest.len() < forward_length {
                        Err(WouldBlock)
                    } else if rest.len() == forward_length {
                        match Command::parse(rest) {
                            Ok(command) => Ok(SecondaryCommand(command).into()),
                            // This will happen if the forward_length byte is corrupt.
                            Err(WouldBlock) => Err(Other(ParseError)),
                            Err(e) => Err(e),
                        }
                    } else {
                        Err(Other(ParseError))
                    }
                } else {
                    Err(WouldBlock)
                }
            }
            _ => Ok(Command::parse(buf)?.into()),
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

    mod message {
        use super::*;

        #[test]
        fn missing_byte_secondary() {
            let message = Message::from(SecondaryCommand(Command::PowerOff));

            let mut buf = vec![];
            message.write_to_std(&mut buf).unwrap();
            for prefix_length in 0..buf.len() {
                let round_tripped_message = Message::parse(&buf[..prefix_length]);
                assert_eq!(round_tripped_message, Err(WouldBlock))
            }
        }

        #[test]
        fn parse_error_if_extra_byte() {
            let message = Message::from(SecondaryCommand(Command::PowerOff));

            let mut buf = vec![];
            message.write_to_std(&mut buf).unwrap();
            buf.push(42);
            let round_tripped_message = Message::parse(&buf);

            assert_eq!(round_tripped_message, Err(Other(ParseError)))
        }

        #[test]
        fn parse_error_if_length_wrong() {
            let message = Message::from(SecondaryCommand(Command::PowerOff));

            let mut buf = vec![];
            message.write_to_std(&mut buf).unwrap();
            buf[1] -= 1;
            buf.pop();
            let round_tripped_message = Message::parse(&buf);

            assert_eq!(round_tripped_message, Err(Other(ParseError)))
        }
    }

    // TODO: see if it's possible to verify this round-trip property
    // for all Command variants using cargo-propverify, so we don't
    // have to maintain this test as we add/remove variants.
    mod round_trip {
        use super::*;
        use test_case::test_case;
        use Command::*;

        #[test_case(SetSideLed(true))]
        #[test_case(SetOrangeLed(false))]
        #[test_case(SetRedLed(true))]
        #[test_case(SetGreenLed(false))]
        #[test_case(SetMaxSpeed(-30..=42))]
        #[test_case(SetSpringConstant(42))]
        #[test_case(SetTarget(-42))]
        #[test_case(Recenter)]
        #[test_case(ReportBattery)]
        #[test_case(ReportCharger)]
        #[test_case(RemoveTarget)]
        #[test_case(IncrementTarget)]
        #[test_case(DecrementTarget)]
        #[test_case(PowerOff)]
        fn round_trip_equality(command: Command) {
            let mut buf = vec![];
            command.write_to_std(&mut buf).unwrap();
            let round_tripped_command = Command::parse(&buf).unwrap();

            assert_eq!(round_tripped_command, command)
        }

        #[test_case(SetSideLed(true))]
        #[test_case(SetOrangeLed(false))]
        #[test_case(SetRedLed(true))]
        #[test_case(SetGreenLed(false))]
        #[test_case(SetMaxSpeed(-30..=42))]
        #[test_case(SetSpringConstant(42))]
        #[test_case(SetTarget(-42))]
        #[test_case(Recenter)]
        #[test_case(ReportBattery)]
        #[test_case(ReportCharger)]
        #[test_case(RemoveTarget)]
        #[test_case(IncrementTarget)]
        #[test_case(DecrementTarget)]
        #[test_case(PowerOff)]
        fn would_block_if_missing_byte(command: Command) {
            let mut buf = vec![];
            command.write_to_std(&mut buf).unwrap();
            for prefix_length in 0..buf.len() {
                let round_tripped_command = Command::parse(&buf[..prefix_length]);
                assert_eq!(round_tripped_command, Err(WouldBlock))
            }
        }

        #[test_case(SetSideLed(true))]
        #[test_case(SetOrangeLed(false))]
        #[test_case(SetRedLed(true))]
        #[test_case(SetGreenLed(false))]
        #[test_case(SetMaxSpeed(-30..=42))]
        #[test_case(SetSpringConstant(42))]
        #[test_case(SetTarget(-42))]
        #[test_case(Recenter)]
        #[test_case(ReportBattery)]
        #[test_case(ReportCharger)]
        #[test_case(RemoveTarget)]
        #[test_case(IncrementTarget)]
        #[test_case(DecrementTarget)]
        #[test_case(PowerOff)]
        fn parse_error_if_extra_byte(command: Command) {
            let mut buf = vec![];
            command.write_to_std(&mut buf).unwrap();
            buf.push(42);
            let round_tripped_command = Command::parse(&buf);

            assert_eq!(round_tripped_command, Err(Other(ParseError)))
        }
    }
}
