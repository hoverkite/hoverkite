use crate::util::{ascii_to_bool, bool_to_ascii};
#[cfg(feature = "std")]
use crate::WriteCompat;
use crate::{ProtocolError, Side};
use arrayvec::ArrayString;
use core::convert::TryInto;
use core::mem::size_of;
use nb::Error::{Other, WouldBlock};

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Response {
    Log(ArrayString<256>),
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

    pub fn parse(buf: &[u8]) -> nb::Result<Self, ProtocolError> {
        let report = match *buf {
            [] => return Err(WouldBlock),
            [b'"', ref rest @ ..] => {
                if let [ref rest @ .., b'\n'] = *rest {
                    let utf8 = core::str::from_utf8(rest).map_err(ProtocolError::Utf8Error)?;
                    let message =
                        ArrayString::from(utf8).map_err(|_| ProtocolError::MessageTooLong)?;
                    Self::Log(message)
                } else {
                    return Err(WouldBlock);
                }
            }
            [b'I', ref rest @ ..] => {
                if rest.len() < size_of::<i64>() {
                    return Err(WouldBlock);
                }
                let bytes = rest
                    .try_into()
                    .map_err(|_| Other(ProtocolError::MessageTooLong))?;
                let position = i64::from_le_bytes(bytes);
                Self::Position(position)
            }
            [b'B', ref rest @ ..] => {
                if rest.len() < 6 {
                    return Err(WouldBlock);
                } else if rest.len() > 6 {
                    return Err(Other(ProtocolError::MessageTooLong));
                }
                let battery_voltage = u16::from_le_bytes(rest[..2].try_into().unwrap());
                let backup_battery_voltage = u16::from_le_bytes(rest[2..4].try_into().unwrap());
                let motor_current = u16::from_le_bytes(rest[4..6].try_into().unwrap());
                Self::BatteryReadings {
                    battery_voltage,
                    backup_battery_voltage,
                    motor_current,
                }
            }
            [b'C'] => return Err(WouldBlock),
            [b'C', charger_connected] => Self::ChargeState {
                charger_connected: ascii_to_bool(charger_connected)?,
            },
            [b'p'] => Self::PowerOff,
            [c, ..] => return Err(Other(ProtocolError::InvalidCommand(c))),
        };
        Ok(report)
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

    pub fn parse(buffer: &[u8]) -> nb::Result<Self, ProtocolError> {
        if let [side, ref rest @ ..] = *buffer {
            Ok(SideResponse {
                side: Side::parse(side)?,
                response: Response::parse(rest)?,
            })
        } else {
            Err(WouldBlock)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use test_case::test_case;

    #[test]
    fn parse_invalid() {
        let mut buffer = Vec::new();
        buffer.extend(b"x");
        assert_eq!(
            SideResponse::parse(&mut buffer),
            Err(Other(ProtocolError::InvalidSide(b'x')))
        );
    }

    #[test]
    fn parse_empty() {
        let mut buffer = Vec::new();
        assert_eq!(SideResponse::parse(&mut buffer), Err(WouldBlock));
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
            let mut buffer = Vec::new();
            buffer.extend(&partial_response[..length]);
            assert_eq!(SideResponse::parse(&mut buffer), Err(WouldBlock));
        }
    }

    #[test]
    fn parse_invalid_charge_state() {
        let mut buffer = Vec::new();
        buffer.extend(b"RCx");
        assert_eq!(
            SideResponse::parse(&mut buffer),
            Err(Other(ProtocolError::InvalidByte(b'x')))
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
        let mut buffer = Vec::new();
        buffer.extend(bytes);
        assert_eq!(
            SideResponse::parse(&mut buffer),
            Ok(SideResponse {
                side: Side::Right,
                response: response,
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

        assert_eq!(SideResponse::parse(&mut buffer), Ok(side_response));
    }
}
