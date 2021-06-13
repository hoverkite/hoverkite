use super::{Command, DirectedCommand, Note, Side, SideResponse, SpeedLimits};
use log::{error, trace};
use serialport::SerialPort;
use slice_deque::SliceDeque;
use std::io;
use std::thread::sleep;
use std::time::{Duration, Instant};

/// The minimum amount of time to wait between sending consecutive target commands to the device, to
/// avoid overwhelming it or overflowing its receive buffer.
pub const MIN_TIME_BETWEEN_TARGET_UPDATES: Duration = Duration::from_millis(100);
const NOTE_SEND_SLEEP_DURATION: Duration = Duration::from_millis(50);

/// A client to talk to a Hoverkite device over one or two serial ports.
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
    right_buffer: SliceDeque<u8>,
    left_buffer: SliceDeque<u8>,
}

impl Hoverkite {
    /// Constructs a new `Hoverkite`, communicating over the given serial ports. If only one of the
    /// serial ports is given then all commands will be sent to it, to be forwarded.
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
            right_buffer: SliceDeque::new(),
            left_buffer: SliceDeque::new(),
        }
    }

    /// Sends any pending target commands, reads from both serial ports, and returns any available
    /// responses.
    pub fn poll(&mut self) -> Result<Vec<SideResponse>, io::Error> {
        self.send_pending_targets()?;

        let mut responses = vec![];
        if let Some(port) = &mut self.left_port {
            responses.extend(read_port(port, &mut self.left_buffer)?);
        }
        if let Some(port) = &mut self.right_port {
            responses.extend(read_port(port, &mut self.right_buffer)?);
        }

        Ok(responses)
    }

    fn send_pending_targets(&mut self) -> Result<(), io::Error> {
        if let Some(target_pending) = self.left_target_pending {
            // Just retry. If the rate limit is still in effect then this will be a no-op.
            self.set_target(Side::Left, target_pending)?;
        }
        if let Some(target_pending) = self.right_target_pending {
            self.set_target(Side::Right, target_pending)?;
        }
        Ok(())
    }

    /// Sets the maximum 'speed' (really more like torque) on both sides.
    pub fn set_max_speed(&mut self, side: Side, max_speed: SpeedLimits) -> Result<(), io::Error> {
        println!("{:?} max speed: {}", side, max_speed);
        let command = Command::SetMaxSpeed(max_speed);
        self.send_command(side, command)?;
        Ok(())
    }

    /// Sets the spring constant to the given value on both sides.
    pub fn set_spring_constant(&mut self, spring_constant: u16) -> Result<(), io::Error> {
        println!("Spring constant: {}", spring_constant);
        let command = Command::SetSpringConstant(spring_constant);
        self.send_command(Side::Left, command.clone())?;
        self.send_command(Side::Right, command)?;
        Ok(())
    }

    /// Plays the given sequence of notes on the hoverboard.
    ///
    /// This method sleeps for a short time between sending each note, to avoid overflowing the
    /// hoverboard's note buffer, so it will take a while.
    pub fn play_notes_blocking(&mut self, notes: &[Note]) -> Result<(), io::Error> {
        for note in notes {
            trace!("Sending {:?}", note);
            let command = Command::AddBuzzerNote(*note);
            self.send_command(Side::Left, command)?;
            sleep(NOTE_SEND_SLEEP_DURATION);
        }
        Ok(())
    }

    /// Sets the given target position.
    ///
    /// These commands are automatically rate-limited, to avoid overflowing the hoverboard's receive
    /// buffer.
    pub fn set_target(&mut self, side: Side, target: i64) -> Result<(), io::Error> {
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
        self.send_command(side, Command::SetTarget(target))
    }

    /// Sends the given command to the given side.
    pub fn send_command(&mut self, side: Side, command: Command) -> Result<(), io::Error> {
        trace!("Sending command to {:?}: {:?}", side, command);
        match side {
            Side::Left => {
                self.left_last_command_time = Instant::now();
            }
            Side::Right => {
                self.right_last_command_time = Instant::now();
            }
        };
        let side_command = DirectedCommand { side, command };
        let port = match (side, self.left_port.as_mut(), self.right_port.as_mut()) {
            (Side::Left, Some(port), _) => port,
            (Side::Left, None, Some(port)) => port,
            (Side::Right, _, Some(port)) => port,
            (Side::Right, Some(port), None) => port,
            (_, None, None) => {
                error!(
                    "No serial ports available. Can't send command {:?}",
                    side_command
                );
                return Ok(());
            }
        };
        side_command.write_to_std(port)?;
        Ok(())
    }
}

fn read_port(
    port: &mut Box<dyn SerialPort>,
    buffer: &mut SliceDeque<u8>,
) -> Result<Option<SideResponse>, io::Error> {
    if port.bytes_to_read()? > 0 {
        let mut temp = [0; 100];
        let bytes_read = port.read(&mut temp)?;
        buffer.extend(&temp[0..bytes_read]);
    }

    match SideResponse::parse(&buffer) {
        Ok((response, len)) => {
            buffer.drain(..len);
            return Ok(Some(response));
        }
        Err(nb::Error::Other((e, len))) => {
            error!("Unexpected response {:?} from {:?}", e, &buffer[..len]);
            buffer.drain(..len);
            return Ok(None);
        }
        Err(nb::Error::WouldBlock) => (),
    }

    Ok(None)
}
