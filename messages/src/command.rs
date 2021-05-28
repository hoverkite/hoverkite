use crate::util::{ascii_to_bool, bool_to_ascii};
#[cfg(feature = "std")]
use crate::WriteCompat;
use crate::{ProtocolError, Side};
use core::convert::TryInto;
use core::fmt::{self, Display, Formatter};
use core::mem::size_of;
use core::num::NonZeroU32;
use core::ops::RangeInclusive;
use nb::Error::{Other, WouldBlock};

/// A note to play on the buzzer.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct Note {
    /// The frequency in Hertz, or `None` for silence.
    pub frequency: Option<NonZeroU32>,
    /// The duration in milliseconds.
    pub duration_ms: u32,
}

impl Display for Note {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        if let Some(frequency) = self.frequency {
            write!(f, "{} Hz for {} ms", frequency, self.duration_ms)
        } else {
            write!(f, "silent for {} ms", self.duration_ms)
        }
    }
}

/// Speed (or really, torque) limits for the motor.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct SpeedLimits {
    /// The lowest negative speed, inclusive.
    pub negative: i16,
    /// The highest positive speed, inclusive.
    pub positive: i16,
}

impl SpeedLimits {
    /// Swap the limits, so e.g. -42..66 becomes -66..42.
    pub fn invert(self) -> Self {
        Self {
            negative: -self.positive,
            positive: -self.negative,
        }
    }
}

impl Display for SpeedLimits {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{}..{}", self.negative, self.positive)
    }
}

impl From<SpeedLimits> for RangeInclusive<i16> {
    fn from(limits: SpeedLimits) -> Self {
        limits.negative..=limits.positive
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Command {
    SetSideLed(bool),
    SetOrangeLed(bool),
    SetRedLed(bool),
    SetGreenLed(bool),
    SetBuzzerFrequency(u32),
    AddBuzzerNote(Note),
    ReportBattery,
    ReportCharger,
    SetMaxSpeed(SpeedLimits),
    SetSpringConstant(u16),
    SetTarget(i64),
    RemoveTarget,
    Recenter,
    IncrementTarget,
    DecrementTarget,
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
            Self::SetSideLed(on) => writer.bwrite_all(&[b'l', bool_to_ascii(*on)])?,
            Self::SetOrangeLed(on) => writer.bwrite_all(&[b'o', bool_to_ascii(*on)])?,
            Self::SetRedLed(on) => writer.bwrite_all(&[b'r', bool_to_ascii(*on)])?,
            Self::SetGreenLed(on) => writer.bwrite_all(&[b'g', bool_to_ascii(*on)])?,
            Self::SetBuzzerFrequency(frequency) => {
                writer.bwrite_all(b"f")?;
                writer.bwrite_all(&frequency.to_le_bytes())?;
            }
            Self::AddBuzzerNote(note) => {
                writer.bwrite_all(b"d")?;
                writer.bwrite_all(&note.frequency.map_or(0, NonZeroU32::get).to_le_bytes())?;
                writer.bwrite_all(&note.duration_ms.to_le_bytes())?;
            }
            Self::SetMaxSpeed(max_speed) => {
                writer.bwrite_all(b"S")?;
                writer.bwrite_all(&max_speed.negative.to_le_bytes())?;
                writer.bwrite_all(&max_speed.positive.to_le_bytes())?;
            }
            Self::SetSpringConstant(spring_constant) => {
                writer.bwrite_all(b"K")?;
                writer.bwrite_all(&spring_constant.to_le_bytes())?;
            }
            Self::SetTarget(target) => {
                writer.bwrite_all(b"T")?;
                writer.bwrite_all(&target.to_le_bytes())?;
            }
            Self::Recenter => writer.bwrite_all(b"e")?,
            Self::ReportBattery => writer.bwrite_all(b"b")?,
            Self::ReportCharger => writer.bwrite_all(b"c")?,
            Self::RemoveTarget => writer.bwrite_all(b"n")?,
            Self::IncrementTarget => writer.bwrite_all(b"+")?,
            Self::DecrementTarget => writer.bwrite_all(b"-")?,
            Self::PowerOff => writer.bwrite_all(b"p")?,
        };
        Ok(())
    }

    pub fn parse(buf: &[u8]) -> nb::Result<Self, ProtocolError> {
        let command = match *buf {
            [] | [b'l'] | [b'o'] | [b'r'] | [b'g'] => return Err(WouldBlock),
            [b'l', on] => Self::SetSideLed(ascii_to_bool(on)?),
            [b'o', on] => Self::SetOrangeLed(ascii_to_bool(on)?),
            [b'r', on] => Self::SetRedLed(ascii_to_bool(on)?),
            [b'g', on] => Self::SetGreenLed(ascii_to_bool(on)?),
            [b'b'] => Self::ReportBattery,
            [b'c'] => Self::ReportCharger,
            [b'f', ref rest @ ..] => {
                if rest.len() < size_of::<u32>() {
                    return Err(WouldBlock);
                }
                let bytes = rest
                    .try_into()
                    .map_err(|_| Other(ProtocolError::MessageTooLong))?;
                let frequency = u32::from_le_bytes(bytes);
                Self::SetBuzzerFrequency(frequency)
            }
            [b'd', ref rest @ ..] => {
                if rest.len() < 8 {
                    return Err(WouldBlock);
                }
                if rest.len() > 8 {
                    return Err(Other(ProtocolError::MessageTooLong));
                }
                let frequency = u32::from_le_bytes(rest[..4].try_into().unwrap());
                let duration_ms = u32::from_le_bytes(rest[4..8].try_into().unwrap());
                Self::AddBuzzerNote(Note {
                    frequency: NonZeroU32::new(frequency),
                    duration_ms,
                })
            }
            [b'S', ref rest @ ..] => {
                if rest.len() < 4 {
                    return Err(WouldBlock);
                }
                if rest.len() > 4 {
                    return Err(Other(ProtocolError::MessageTooLong));
                }
                let negative = i16::from_le_bytes(rest[..2].try_into().unwrap());
                let positive = i16::from_le_bytes(rest[2..4].try_into().unwrap());
                Self::SetMaxSpeed(SpeedLimits { negative, positive })
            }
            [b'K', ref rest @ ..] => {
                if rest.len() < size_of::<u16>() {
                    return Err(WouldBlock);
                }
                let bytes = rest
                    .try_into()
                    .map_err(|_| Other(ProtocolError::MessageTooLong))?;
                let spring = u16::from_le_bytes(bytes);
                Self::SetSpringConstant(spring)
            }
            [b'n'] => Self::RemoveTarget,
            [b'T', ref rest @ ..] => {
                if rest.len() < size_of::<i64>() {
                    return Err(WouldBlock);
                }
                let bytes = rest
                    .try_into()
                    .map_err(|_| Other(ProtocolError::MessageTooLong))?;
                let target = i64::from_le_bytes(bytes);
                Self::SetTarget(target)
            }
            [b'e'] => Self::Recenter,
            [b'+'] => Self::IncrementTarget,
            [b'-'] => Self::DecrementTarget,
            [b'p'] => Self::PowerOff,
            [c] => return Err(Other(ProtocolError::InvalidCommand(c))),
            [..] => return Err(Other(ProtocolError::MessageTooLong)),
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
    pub fn parse(buf: &[u8]) -> nb::Result<Self, ProtocolError> {
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

    mod speed_limits {
        use super::*;

        #[test]
        fn display() {
            assert_eq!(
                SpeedLimits {
                    negative: -42,
                    positive: 66
                }
                .to_string(),
                "-42..66"
            )
        }

        #[test]
        fn to_range_inclusive() {
            let range: RangeInclusive<i16> = SpeedLimits {
                negative: -42,
                positive: 66,
            }
            .into();
            assert_eq!(range, -42..=66)
        }
    }

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

            assert_eq!(
                round_tripped_command,
                Err(Other(ProtocolError::MessageTooLong))
            )
        }

        #[test]
        fn parse_error_if_bogus_payload() {
            assert_eq!(
                DirectedCommand::parse(&[b'R', b'!']),
                Err(Other(ProtocolError::InvalidCommand(b'!')))
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
        #[test_case(SetBuzzerFrequency(42))]
        #[test_case(AddBuzzerNote(Note { frequency: NonZeroU32::new(123), duration_ms: 456 }))]
        #[test_case(AddBuzzerNote(Note { frequency: None, duration_ms: 456 }))]
        #[test_case(SetMaxSpeed(SpeedLimits { negative: -30, positive: 42 }))]
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
        #[test_case(SetBuzzerFrequency(42))]
        #[test_case(AddBuzzerNote(Note { frequency: NonZeroU32::new(123), duration_ms: 456 }))]
        #[test_case(AddBuzzerNote(Note { frequency: None, duration_ms: 456 }))]
        #[test_case(SetMaxSpeed(SpeedLimits { negative: -30, positive: 42 }))]
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

            assert_eq!(
                round_tripped_command,
                Err(Other(ProtocolError::MessageTooLong))
            )
        }

        #[test_case(SetSideLed(true))]
        #[test_case(SetOrangeLed(false))]
        #[test_case(SetRedLed(true))]
        #[test_case(SetGreenLed(false))]
        #[test_case(SetBuzzerFrequency(42))]
        #[test_case(AddBuzzerNote(Note { frequency: NonZeroU32::new(123), duration_ms: 456 }))]
        #[test_case(AddBuzzerNote(Note { frequency: None, duration_ms: 456 }))]
        #[test_case(SetMaxSpeed(SpeedLimits { negative: -30, positive: 42 }))]
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
