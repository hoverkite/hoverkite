use eyre::Report;
use hoverkite_protocol::{Command, DirectedCommand, Side};
use log::{error, trace};
use serialport::SerialPort;
use std::convert::TryInto;
use std::time::{Duration, Instant};
use std::{collections::VecDeque, ops::RangeInclusive};

pub const MIN_TIME_BETWEEN_TARGET_UPDATES: Duration = Duration::from_millis(100);

pub struct Hoverkite {
    right_port: Option<Box<dyn SerialPort>>,
    left_port: Option<Box<dyn SerialPort>>,
    /// The time that the last command was sent to the right port.
    right_last_command_time: Instant,
    /// The time that the last command was sent to the left port.
    left_last_command_time: Instant,
    /// A pending target command that still needs to be sent but wasn't because of the minimum time
    /// between updates.
    right_target_pending: Option<i64>,
    left_target_pending: Option<i64>,
    right_buffer: VecDeque<u8>,
    left_buffer: VecDeque<u8>,
}

impl Hoverkite {
    pub fn new(
        right_port: Option<Box<dyn SerialPort>>,
        left_port: Option<Box<dyn SerialPort>>,
    ) -> Self {
        Self {
            right_port,
            left_port,
            right_last_command_time: Instant::now(),
            left_last_command_time: Instant::now(),
            right_target_pending: None,
            left_target_pending: None,
            right_buffer: VecDeque::new(),
            left_buffer: VecDeque::new(),
        }
    }

    pub fn process(&mut self) -> Result<(), Report> {
        self.send_pending_targets()?;

        if let Some(port) = &mut self.left_port {
            let response = read_port(port, &mut self.left_buffer, Side::Left)?;
            print_response(&response);
        }
        if let Some(port) = &mut self.right_port {
            let response = read_port(port, &mut self.right_buffer, Side::Right)?;
            print_response(&response);
        }

        Ok(())
    }

    fn send_pending_targets(&mut self) -> Result<(), Report> {
        if let Some(target_pending) = self.left_target_pending {
            // Just retry. If the rate limit is still in effect then this will be a no-op.
            self.set_target(Side::Left, target_pending)?;
        }
        if let Some(target_pending) = self.right_target_pending {
            self.set_target(Side::Right, target_pending)?;
        }
        Ok(())
    }

    pub fn set_max_speed(
        &mut self,
        side: Side,
        max_speed: &RangeInclusive<i16>,
    ) -> Result<(), Report> {
        println!("{:?} max speed: {:?}", side, max_speed);
        let command = Command::SetMaxSpeed(max_speed.clone());
        self.send_command(side, command)?;
        Ok(())
    }

    pub fn set_spring_constant(&mut self, spring_constant: u16) -> Result<(), Report> {
        println!("Spring constant: {}", spring_constant);
        let command = Command::SetSpringConstant(spring_constant);
        self.send_command(Side::Left, command.clone())?;
        self.send_command(Side::Right, command)?;
        Ok(())
    }

    /// Set the given target position.
    ///
    /// These commands are automatically rate-limited, to avoid overflowing the hoverboard's receive
    /// buffer.
    pub fn set_target(&mut self, side: Side, target: i64) -> Result<(), Report> {
        let now = Instant::now();
        match side {
            Side::Left => {
                if now < self.left_last_command_time + MIN_TIME_BETWEEN_TARGET_UPDATES {
                    self.left_target_pending = Some(target);
                    return Ok(());
                } else {
                    self.left_target_pending = None;
                }
            }
            Side::Right => {
                if now < self.right_last_command_time + MIN_TIME_BETWEEN_TARGET_UPDATES {
                    self.right_target_pending = Some(target);
                    return Ok(());
                } else {
                    self.right_target_pending = None;
                }
            }
        };
        println!("Target {:?} {}", side, target);
        let mut command = vec![b'T'];
        command.extend_from_slice(&target.to_le_bytes());
        self.send_command(side, Command::SetTarget(target))
    }
    pub fn send_command(&mut self, side: Side, command: Command) -> Result<(), Report> {
        let port = match side {
            Side::Left => {
                self.left_last_command_time = Instant::now();
                &mut self.left_port
            }
            Side::Right => {
                self.right_last_command_time = Instant::now();
                &mut self.right_port
            }
        };
        trace!("Sending command to {:?}: {:?}", side, command);
        if let Some(port) = port {
            command.write_to(port)?;
        } else if side == Side::Left {
            // Tell the right side to forward the command to the left side.
            let port = self.right_port.as_mut().unwrap();
            DirectedCommand::Left(command).write_to(port)?;
        }
        Ok(())
    }
}

fn print_response(response: &Option<Response>) {
    match response {
        Some(Response {
            side,
            response: SideResponse::Log(log),
        }) => println!("{:?}: '{}'", side, log),
        Some(Response {
            side,
            response: SideResponse::Position(position),
        }) => println!("{:?} at {}", side, position),
        None => {}
    }
}

fn read_port(
    port: &mut Box<dyn SerialPort>,
    buffer: &mut VecDeque<u8>,
    side: Side,
) -> Result<Option<Response>, Report> {
    if port.bytes_to_read()? > 0 {
        let mut temp = [0; 100];
        let bytes_read = port.read(&mut temp)?;
        buffer.extend(&temp[0..bytes_read]);
    }

    Ok(parse_response(buffer, side))
}

fn parse_response(buffer: &mut VecDeque<u8>, side: Side) -> Option<Response> {
    match buffer.front() {
        Some(b'"') | Some(b'\'') => {
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
                Some(Response {
                    side,
                    response: SideResponse::Log(string.into_owned()),
                })
            } else {
                None
            }
        }
        Some(b'I') | Some(b'i') => {
            if buffer.len() >= 9 {
                let side = if buffer.pop_front().unwrap() == b'I' {
                    side
                } else {
                    side.opposite()
                };
                let bytes: Vec<u8> = buffer.drain(0..8).collect();
                let position = i64::from_le_bytes(bytes.try_into().unwrap());
                Some(Response {
                    side,
                    response: SideResponse::Position(position),
                })
            } else {
                None
            }
        }
        Some(r) => {
            error!("Unexpected response {:?}", r);
            buffer.pop_front();
            None
        }
        None => None,
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Response {
    pub side: Side,
    pub response: SideResponse,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SideResponse {
    Log(String),
    Position(i64),
}
