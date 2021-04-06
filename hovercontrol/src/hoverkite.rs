use eyre::Report;
use log::trace;
use serialport::SerialPort;
use std::time::{Duration, Instant};

pub const MIN_TIME_BETWEEN_TARGET_UPDATES: Duration = Duration::from_millis(100);

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Side {
    Left,
    Right,
}

pub struct Hoverkite {
    left_port: Option<Box<dyn SerialPort>>,
    right_port: Option<Box<dyn SerialPort>>,
    /// The time that the last command was sent to the left port.
    left_last_command_time: Instant,
    /// The time that the last command was sent to the right port.
    right_last_command_time: Instant,
    /// A pending target command that still needs to be sent but wasn't because of the minimum time
    /// between updates.
    left_target_pending: Option<i64>,
    right_target_pending: Option<i64>,
    left_buffer: [u8; 100],
    right_buffer: [u8; 100],
    left_length: usize,
    right_length: usize,
}

impl Hoverkite {
    pub fn new(
        left_port: Option<Box<dyn SerialPort>>,
        right_port: Option<Box<dyn SerialPort>>,
    ) -> Self {
        Self {
            left_port,
            right_port,
            left_last_command_time: Instant::now(),
            right_last_command_time: Instant::now(),
            left_target_pending: None,
            right_target_pending: None,
            left_buffer: [0; 100],
            right_buffer: [0; 100],
            left_length: 0,
            right_length: 0,
        }
    }

    pub fn process(&mut self) -> Result<(), Report> {
        self.send_pending_targets()?;

        if let Some(port) = &mut self.left_port {
            read_port(port, &mut self.left_buffer, &mut self.left_length, "Left")?;
        }
        if let Some(port) = &mut self.right_port {
            read_port(
                port,
                &mut self.right_buffer,
                &mut self.right_length,
                "Right",
            )?;
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

    pub fn set_max_speed(&mut self, max_speed: i16) -> Result<(), Report> {
        println!("Max speed: {}", max_speed);
        let mut command = vec![b'S'];
        command.extend_from_slice(&max_speed.to_le_bytes());
        self.send_command(Side::Left, &command)?;
        self.send_command(Side::Right, &command)?;
        Ok(())
    }

    pub fn set_spring_constant(&mut self, spring_constant: u16) -> Result<(), Report> {
        println!("Spring constant: {}", spring_constant);
        let mut command = vec![b'K'];
        command.extend_from_slice(&spring_constant.to_le_bytes());
        self.send_command(Side::Left, &command)?;
        self.send_command(Side::Right, &command)?;
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
        self.send_command(side, &command)
    }

    /// Send the given raw command.
    pub fn send_command(&mut self, side: Side, command: &[u8]) -> Result<(), Report> {
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
        trace!("Sending command: {:?}", command);
        if let Some(port) = port {
            port.write_all(command)?;
        }
        Ok(())
    }
}

fn read_port(
    port: &mut Box<dyn SerialPort>,
    buffer: &mut [u8],
    length: &mut usize,
    name: &str,
) -> Result<(), Report> {
    if port.bytes_to_read()? > 0 {
        *length += port.read(&mut buffer[*length..])?;
        if let Some(end_of_line) = buffer[0..*length].iter().position(|&c| c == b'\n') {
            let string = String::from_utf8_lossy(&buffer[0..end_of_line]);
            println!("{}: '{}'", name, string);
            let remaining_length = *length - end_of_line - 1;
            let remaining_bytes = buffer[end_of_line + 1..*length].to_owned();
            buffer[0..remaining_length].clone_from_slice(&remaining_bytes);
            *length = remaining_length;
        } else if *length == buffer.len() {
            let string = String::from_utf8_lossy(&buffer[0..*length]);
            println!("{}: '{}'", name, string);
            *length = 0;
        }
    }
    Ok(())
}
