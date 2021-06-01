use crate::util::{ascii_to_bool, bool_to_ascii};
#[cfg(feature = "std")]
use crate::WriteCompat;
use crate::{ProtocolError, Side};
use arrayvec::ArrayString;
use core::mem::size_of;
use core::{convert::TryInto, fmt::Write, str};
use nb::Error::{Other, WouldBlock};

const MAX_LOG_SIZE: usize = 256;

struct TruncatingWriter(ArrayString<MAX_LOG_SIZE>);

impl Write for TruncatingWriter {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        if self.0.write_str(s).is_err() {
            // ArrayString::write_str() is atomic - it will refuse to write
            // anything at all, if it finds that the string is too long.
            // If it fails, we unpack into chars and write as many as we can.
            s.chars().try_for_each(|c| self.0.write_char(c))?
        }

        Ok(())
    }

    fn write_fmt(&mut self, fmt: core::fmt::Arguments<'_>) -> core::fmt::Result {
        if core::fmt::write(self, fmt).is_ok() {
            return Ok(());
        }

        // `core::fmt::Error` doesn't have a payload, so we just have to guess.
        if self.0.len() + size_of::<char>() >= MAX_LOG_SIZE {
            // If we think we ran out of bytes while writing, truncate with ...
            self.0.pop();
            self.0.pop();
            self.0.pop();
            self.0.write_str("...")
        } else {
            Err(core::fmt::Error)
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Response {
    Log(ArrayString<MAX_LOG_SIZE>),
    Position(i64),
    BatteryReadings {
        battery_voltage: u16,
        backup_battery_voltage: u16,
        motor_current: u16,
    },
    ChargeState {
        charger_connected: bool,
    },
    PowerOff,
}

impl Response {
    pub fn log_from_fmt(args: core::fmt::Arguments<'_>) -> Self {
        let mut writer = TruncatingWriter(ArrayString::new());
        writer.write_fmt(args).unwrap();

        Self::Log(writer.0)
    }

    pub fn write_to<W>(&self, writer: &mut W) -> Result<(), W::Error>
    where
        W: embedded_hal::blocking::serial::Write<u8>,
    {
        match self {
            Self::Log(message) => {
                writer.bwrite_all(b"\"")?;
                writer.bwrite_all(message.as_bytes())?;
                writer.bwrite_all(b"\n")
            }
            Self::Position(position) => {
                writer.bwrite_all(b"I")?;
                writer.bwrite_all(&position.to_le_bytes())
            }
            Self::BatteryReadings {
                battery_voltage,
                backup_battery_voltage,
                motor_current,
            } => {
                writer.bwrite_all(b"B")?;
                writer.bwrite_all(&battery_voltage.to_le_bytes())?;
                writer.bwrite_all(&backup_battery_voltage.to_le_bytes())?;
                writer.bwrite_all(&motor_current.to_le_bytes())
            }
            Self::ChargeState { charger_connected } => {
                writer.bwrite_all(&[b'C', bool_to_ascii(*charger_connected)])
            }
            Self::PowerOff => writer.bwrite_all(b"p"),
        }
    }

    pub fn parse(buf: &[u8]) -> nb::Result<(Self, usize), (ProtocolError, usize)> {
        let result: (Response, usize) = match *buf {
            [] => return Err(WouldBlock),
            [b'"', ref rest @ ..] => {
                if let [ref rest @ .., b'\n'] = *rest {
                    let utf8 = str::from_utf8(rest)
                        .map_err(|e| (ProtocolError::Utf8Error(e), rest.len() + 2))?;
                    let message = ArrayString::from(utf8)
                        .map_err(|_| (ProtocolError::MessageTooLong, rest.len() + 2))?;
                    (Self::Log(message), rest.len() + 2)
                } else if rest.len() > MAX_LOG_SIZE {
                    return Err(Other((ProtocolError::MessageTooLong, rest.len() + 1)));
                } else {
                    return Err(WouldBlock);
                }
            }
            [b'I', ref rest @ ..] => {
                if rest.len() < size_of::<i64>() {
                    return Err(WouldBlock);
                }
                let bytes = rest[..8].try_into().unwrap();
                let position = i64::from_le_bytes(bytes);
                (Self::Position(position), 9)
            }
            [b'B', ref rest @ ..] => {
                #[allow(clippy::comparison_chain)]
                if rest.len() < 6 {
                    return Err(WouldBlock);
                }
                let battery_voltage = u16::from_le_bytes(rest[..2].try_into().unwrap());
                let backup_battery_voltage = u16::from_le_bytes(rest[2..4].try_into().unwrap());
                let motor_current = u16::from_le_bytes(rest[4..6].try_into().unwrap());
                (
                    Self::BatteryReadings {
                        battery_voltage,
                        backup_battery_voltage,
                        motor_current,
                    },
                    7,
                )
            }
            [b'C'] => return Err(WouldBlock),
            [b'C', charger_connected, ..] => (
                Self::ChargeState {
                    charger_connected: ascii_to_bool(charger_connected).map_err(|e| (e, 2))?,
                },
                2,
            ),
            [b'p', ..] => (Self::PowerOff, 1),
            [c, ..] => return Err(Other((ProtocolError::InvalidCommand(c), 1))),
        };
        Ok(result)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SideResponse {
    pub side: Side,
    pub response: Response,
}

impl SideResponse {
    #[cfg(feature = "std")]
    pub fn write_to_std(&self, writer: impl std::io::Write) -> std::io::Result<()> {
        self.write_to(&mut WriteCompat(writer))
    }

    pub fn write_to<W>(&self, writer: &mut W) -> Result<(), W::Error>
    where
        W: embedded_hal::blocking::serial::Write<u8>,
    {
        writer.bwrite_all(&[self.side.to_byte()])?;
        self.response.write_to(writer)
    }

    pub fn parse_exact(buffer: &[u8]) -> nb::Result<Self, ProtocolError> {
        match Self::parse(buffer) {
            Ok((result, length)) => {
                if length == buffer.len() {
                    Ok(result)
                } else {
                    Err(Other(ProtocolError::MessageTooLong))
                }
            }
            Err(WouldBlock) => Err(WouldBlock),
            Err(Other((e, _))) => Err(Other(e)),
        }
    }

    pub fn parse(buffer: &[u8]) -> nb::Result<(Self, usize), (ProtocolError, usize)> {
        if let [side, ref rest @ ..] = *buffer {
            let side = Side::parse(side).map_err(|e| (e, 1))?;
            match Response::parse(rest) {
                Ok((response, length)) => Ok((SideResponse { side, response }, length + 1)),
                Err(WouldBlock) => Err(WouldBlock),
                Err(Other((e, length))) => Err(Other((e, length + 1))),
            }
        } else {
            Err(WouldBlock)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use test_case::test_case;

    mod log {
        use super::super::*;
        use test_case::test_case;

        #[test_case("$" ; "width 1")]
        #[test_case("¢" ; "width 2")]
        #[test_case("€" ; "width 3")]
        #[test_case("𐍈" ; "width 4")]
        fn too_long_with_unicode_widths(c: &str) {
            let response = Response::log_from_fmt(format_args!("{}", c.repeat(500)));
            let log = match response {
                Response::Log(log) => log,
                _ => panic!(),
            };
            assert!(&log[..].ends_with("..."))
        }

        #[test]
        fn parse_too_long() {
            let buf = format!("\"{}\n", "n".repeat(500));
            let response = Response::parse(buf.as_bytes());
            assert_eq!(response, Err(Other((ProtocolError::MessageTooLong, 502))));
        }
    }

    #[test]
    fn parse_invalid_side() {
        assert_eq!(
            SideResponse::parse(b"x"),
            Err(Other((ProtocolError::InvalidSide(b'x'), 1)))
        );
    }

    #[test]
    fn parse_invalid_command() {
        assert_eq!(
            SideResponse::parse(b"Lx"),
            Err(Other((ProtocolError::InvalidCommand(b'x'), 2)))
        );
    }

    #[test]
    fn parse_empty() {
        assert_eq!(SideResponse::parse(b""), Err(WouldBlock));
    }

    #[test_case(b"RI" ; "position")]
    #[test_case(b"LI" ; "other side position")]
    #[test_case(b"RB12345" ; "battery readings")]
    #[test_case(b"LB12345" ; "other side battery readings")]
    #[test_case(b"RC" ; "charge state")]
    #[test_case(b"LC" ; "other side charge state")]
    #[test_case(b"R\"blah" ; "log")]
    #[test_case(b"L\"blah" ; "other side log")]
    fn parse_partial(partial_response: &[u8]) {
        for length in 1..=partial_response.len() {
            assert_eq!(
                SideResponse::parse(&partial_response[..length]),
                Err(WouldBlock)
            );
        }
    }

    #[test]
    fn parse_invalid_charge_state() {
        assert_eq!(
            SideResponse::parse(b"RCx"),
            Err(Other((ProtocolError::InvalidByte(b'x'), 3)))
        );
    }

    #[test_case(&[b'R', b'I', 0x88, 0x77, 0x66, 0x55, 0x44, 0x33, 0x22, 0x11], Response::Position(0x1122334455667788))]
    #[test_case(&[b'R', b'B', 0x66, 0x55, 0x44, 0x33, 0x22, 0x11], Response::BatteryReadings {
        battery_voltage: 0x5566,
        backup_battery_voltage: 0x3344,
        motor_current: 0x1122,
    })]
    #[test_case(b"RC0", Response::ChargeState { charger_connected: false })]
    #[test_case(b"RC1", Response::ChargeState { charger_connected: true })]
    #[test_case(b"R\"hello\n", Response::Log(ArrayString::from("hello").unwrap()))]
    fn parse_valid(bytes: &[u8], response: Response) {
        assert_eq!(
            SideResponse::parse(bytes),
            Ok((
                SideResponse {
                    side: Side::Right,
                    response: response.clone(),
                },
                bytes.len()
            ))
        );
        assert_eq!(
            SideResponse::parse_exact(bytes),
            Ok(SideResponse {
                side: Side::Right,
                response,
            })
        );
    }

    #[test_case(Response::Position(0x1122334455667788))]
    #[test_case(Response::BatteryReadings {
        battery_voltage: 0x5566,
        backup_battery_voltage: 0x3344,
        motor_current: 0x1122,
    })]
    #[test_case(Response::ChargeState { charger_connected: false })]
    #[test_case(Response::ChargeState { charger_connected: true })]
    #[test_case(Response::Log(ArrayString::from("hello").unwrap()))]
    #[test_case(Response::Log(ArrayString::from("emoji 👨‍👨‍👦").unwrap()))]
    #[test_case(Response::PowerOff)]
    fn round_trip(response: Response) {
        let side_response = SideResponse {
            side: Side::Right,
            response,
        };
        let mut buffer = Vec::new();
        side_response.write_to_std(&mut buffer).unwrap();

        assert_eq!(SideResponse::parse_exact(&mut buffer), Ok(side_response));
    }

    // Note that Log only currently looks at the last byte for `\n`,
    // to avoid quadratic performance when parsing a byte at a time,
    // so it won't detect MessageTooLong until the length exceeds MAX_LOG_SIZE.
    #[test_case(Response::Position(0x1122334455667788))]
    #[test_case(Response::BatteryReadings {
        battery_voltage: 0x5566,
        backup_battery_voltage: 0x3344,
        motor_current: 0x1122,
    })]
    #[test_case(Response::ChargeState { charger_connected: false })]
    #[test_case(Response::ChargeState { charger_connected: true })]
    #[test_case(Response::PowerOff)]
    fn parse_error_if_extra_byte(response: Response) {
        let side_response = SideResponse {
            side: Side::Right,
            response,
        };
        let mut buffer = Vec::new();
        side_response.write_to_std(&mut buffer).unwrap();
        buffer.push(42);

        assert_eq!(
            SideResponse::parse_exact(&mut buffer),
            Err(Other(ProtocolError::MessageTooLong))
        )
    }
}
