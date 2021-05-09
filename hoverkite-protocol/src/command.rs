#[cfg(feature = "std")]
use crate::WriteCompat;
use crate::{ParseError, Side};
use core::mem::size_of;
use core::{convert::TryInto, ops::RangeInclusive};
use nb::Error::{Other, WouldBlock};

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

/// A command for a particular side.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DirectedCommand {
    pub side: Side,
    pub command: Command,
}

impl DirectedCommand {
    pub fn parse(buf: &[u8]) -> nb::Result<Self, ParseError> {
        if let [side, ref rest @ ..] = *buf {
            Ok(DirectedCommand {
                side: Side::parse(side)?,
                command: Command::parse(rest)?,
            })
        } else {
            Err(WouldBlock)
        }
    }

    pub fn write_to<W>(&self, writer: &mut W) -> Result<(), W::Error>
    where
        W: embedded_hal::blocking::serial::Write<u8>,
    {
        writer.bwrite_all(&[self.side.to_byte()])?;
        self.command.write_to(writer)
    }

    // FIXME: This goes away once the blanket impl exists.
    #[cfg(feature = "std")]
    pub fn write_to_std(&self, writer: impl std::io::Write) -> std::io::Result<()> {
        self.write_to(&mut WriteCompat(writer))
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

    mod side_command {
        use super::*;

        #[test]
        fn power_off_left() {
            let command = DirectedCommand {
                side: Side::Left,
                command: Command::PowerOff,
            };
            let mut buf = vec![];
            command.write_to_std(&mut buf).unwrap();
            assert_eq!(buf, b"Lp");
        }

        #[test]
        fn missing_byte_left() {
            let command = DirectedCommand {
                side: Side::Left,
                command: Command::PowerOff,
            };

            let mut buf = vec![];
            command.write_to_std(&mut buf).unwrap();
            for prefix_length in 0..buf.len() {
                let round_tripped_command = DirectedCommand::parse(&buf[..prefix_length]);
                assert_eq!(round_tripped_command, Err(WouldBlock))
            }
        }

        #[test]
        fn parse_error_if_extra_byte() {
            let command = DirectedCommand {
                side: Side::Left,
                command: Command::PowerOff,
            };

            let mut buf = vec![];
            command.write_to_std(&mut buf).unwrap();
            buf.push(42);
            let round_tripped_command = DirectedCommand::parse(&buf);

            assert_eq!(round_tripped_command, Err(Other(ParseError)))
        }

        #[test]
        fn parse_error_if_bogus_payload() {
            assert_eq!(
                DirectedCommand::parse(&[b'R', b'!']),
                Err(Other(ParseError))
            )
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
            let command = DirectedCommand {
                side: Side::Left,
                command,
            };
            let mut buf = vec![];
            command.write_to_std(&mut buf).unwrap();
            let round_tripped_command = DirectedCommand::parse(&buf).unwrap();

            assert_eq!(round_tripped_command, command)
        }
    }
}
