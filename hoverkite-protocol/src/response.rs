use crate::Side;
#[cfg(feature = "std")]
use std::{collections::VecDeque, convert::TryInto};

#[cfg(feature = "std")]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Response {
    pub side: Side,
    pub response: SideResponse,
}

#[cfg(feature = "std")]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SideResponse {
    Log(String),
    Report(SideReport),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SideReport {
    Position(i64),
    BatteryReadings {
        battery_voltage: u16,
        backup_battery_voltage: u16,
        motor_current: u16,
    },
    ChargeState {
        charger_connected: bool,
    },
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct UnexpectedResponse(pub u8);

#[cfg(feature = "std")]
impl Response {
    pub fn parse(
        buffer: &mut VecDeque<u8>,
        side: Side,
    ) -> Result<Option<Self>, UnexpectedResponse> {
        match buffer.front().copied() {
            Some(b'"') | Some(b'\'') => {
                Ok(
                    if let Some(end_of_line) = buffer.iter().position(|&c| c == b'\n') {
                        let side = if buffer.pop_front().unwrap() == b'"' {
                            side
                        } else {
                            side.opposite()
                        };
                        let log: Vec<u8> = buffer.drain(0..end_of_line - 1).collect();
                        // Drop '\n'
                        buffer.pop_front();
                        let string = String::from_utf8_lossy(&log);
                        Some(Self {
                            side,
                            response: SideResponse::Log(string.into_owned()),
                        })
                    } else {
                        None
                    },
                )
            }
            Some(b'I') | Some(b'i') => Ok(if buffer.len() >= 9 {
                let side = if buffer.pop_front().unwrap() == b'I' {
                    side
                } else {
                    side.opposite()
                };
                let bytes: Vec<u8> = buffer.drain(0..8).collect();
                let position = i64::from_le_bytes(bytes.try_into().unwrap());
                Some(Self {
                    side,
                    response: SideResponse::Report(SideReport::Position(position)),
                })
            } else {
                None
            }),
            Some(b'B') | Some(b'b') => Ok(if buffer.len() >= 7 {
                let side = if buffer.pop_front().unwrap() == b'B' {
                    side
                } else {
                    side.opposite()
                };
                let bytes: Vec<u8> = buffer.drain(0..6).collect();
                let battery_voltage = u16::from_le_bytes(bytes[0..2].try_into().unwrap());
                let backup_battery_voltage = u16::from_le_bytes(bytes[2..4].try_into().unwrap());
                let motor_current = u16::from_le_bytes(bytes[4..6].try_into().unwrap());
                Some(Self {
                    side,
                    response: SideResponse::Report(SideReport::BatteryReadings {
                        battery_voltage,
                        backup_battery_voltage,
                        motor_current,
                    }),
                })
            } else {
                None
            }),
            Some(b'C') | Some(b'c') => Ok(if buffer.len() >= 2 {
                let side = if buffer.pop_front().unwrap() == b'C' {
                    side
                } else {
                    side.opposite()
                };
                let byte = buffer.pop_front().unwrap();
                let charger_connected = match byte {
                    b'0' => false,
                    b'1' => true,
                    _ => return Err(UnexpectedResponse(byte)),
                };
                Some(Self {
                    side,
                    response: SideResponse::Report(SideReport::ChargeState { charger_connected }),
                })
            } else {
                None
            }),
            Some(r) => {
                buffer.pop_front();
                Err(UnexpectedResponse(r))
            }
            None => Ok(None),
        }
    }
}

#[cfg(feature = "std")]
#[cfg(test)]
mod tests {
    use super::*;
    use test_case::test_case;

    #[test]
    fn parse_invalid() {
        let mut buffer = VecDeque::new();
        buffer.extend(b"x");
        assert_eq!(
            Response::parse(&mut buffer, Side::Right),
            Err(UnexpectedResponse(b'x'))
        );
        assert_eq!(buffer.len(), 0);
    }

    #[test]
    fn parse_empty() {
        let mut buffer = VecDeque::new();
        assert_eq!(Response::parse(&mut buffer, Side::Right), Ok(None));
        assert_eq!(buffer.len(), 0);
    }

    #[test_case(b"I" ; "position")]
    #[test_case(b"i" ; "other side position")]
    #[test_case(b"B12345" ; "battery readings")]
    #[test_case(b"b12345" ; "other side battery readings")]
    #[test_case(b"C" ; "charge state")]
    #[test_case(b"c" ; "other side charge state")]
    #[test_case(b"\"blah" ; "log")]
    #[test_case(b"'blah" ; "other side log")]
    fn parse_partial(partial_response: &[u8]) {
        for length in 1..=partial_response.len() {
            let mut buffer = VecDeque::new();
            buffer.extend(&partial_response[..length]);
            assert_eq!(Response::parse(&mut buffer, Side::Right), Ok(None));
            // No bytes should be consumed from the buffer.
            assert_eq!(buffer.len(), length);
        }
    }

    #[test]
    fn parse_invalid_charge_state() {
        let mut buffer = VecDeque::new();
        buffer.extend(b"Cx");
        assert_eq!(
            Response::parse(&mut buffer, Side::Right),
            Err(UnexpectedResponse(b'x'))
        );
        assert_eq!(buffer.len(), 0);
    }

    #[test_case(&[b'I', 0x88, 0x77, 0x66, 0x55, 0x44, 0x33, 0x22, 0x11], SideReport::Position(0x1122334455667788))]
    #[test_case(&[b'B', 0x66, 0x55, 0x44, 0x33, 0x22, 0x11], SideReport::BatteryReadings {
        battery_voltage: 0x5566,
        backup_battery_voltage: 0x3344,
        motor_current: 0x1122,
    })]
    #[test_case(b"C0", SideReport::ChargeState { charger_connected: false })]
    #[test_case(b"C1", SideReport::ChargeState { charger_connected: true })]
    fn parse_valid(bytes: &[u8], report: SideReport) {
        let mut buffer = VecDeque::new();
        buffer.extend(bytes);
        assert_eq!(
            Response::parse(&mut buffer, Side::Right),
            Ok(Some(Response {
                side: Side::Right,
                response: SideResponse::Report(report)
            }))
        );
        assert_eq!(buffer.len(), 0);
    }
}
