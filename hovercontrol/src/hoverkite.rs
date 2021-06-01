use log::{error, trace};
use messages::{Command, DirectedCommand, Note, Response, Side, SideResponse, SpeedLimits};
use serialport::SerialPort;
use slice_deque::SliceDeque;
use std::time::{Duration, Instant};

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
    right_buffer: SliceDeque<u8>,
    left_buffer: SliceDeque<u8>,
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
            right_buffer: SliceDeque::new(),
            left_buffer: SliceDeque::new(),
        }
    }

    pub fn process(&mut self) -> Result<(), eyre::Report> {
        self.send_pending_targets()?;

        if let Some(port) = &mut self.left_port {
            let response = read_port(port, &mut self.left_buffer)?;
            print_response(&response);
        }
        if let Some(port) = &mut self.right_port {
            let response = read_port(port, &mut self.right_buffer)?;
            print_response(&response);
        }

        Ok(())
    }

    fn send_pending_targets(&mut self) -> Result<(), eyre::Report> {
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
        max_speed: SpeedLimits,
    ) -> Result<(), eyre::Report> {
        println!("{:?} max speed: {}", side, max_speed);
        let command = Command::SetMaxSpeed(max_speed);
        self.send_command(side, command)?;
        Ok(())
    }

    pub fn set_spring_constant(&mut self, spring_constant: u16) -> Result<(), eyre::Report> {
        println!("Spring constant: {}", spring_constant);
        let command = Command::SetSpringConstant(spring_constant);
        self.send_command(Side::Left, command.clone())?;
        self.send_command(Side::Right, command)?;
        Ok(())
    }

    pub fn set_buzzer_frequency(&mut self, frequency: Option<u32>) -> Result<(), eyre::Report> {
        let command = Command::SetBuzzerFrequency(frequency.unwrap_or(0));
        self.send_command(Side::Left, command)
    }

    /// Play the given sequence of notes on the hoverboard.
    pub fn play_notes(&mut self, notes: &[Note]) -> Result<(), eyre::Report> {
        for note in notes {
            let command = Command::AddBuzzerNote(*note);
            self.send_command(Side::Left, command)?;
        }
        Ok(())
    }

    /// Set the given target position.
    ///
    /// These commands are automatically rate-limited, to avoid overflowing the hoverboard's receive
    /// buffer.
    pub fn set_target(&mut self, side: Side, target: i64) -> Result<(), eyre::Report> {
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

    pub fn send_command(&mut self, side: Side, command: Command) -> Result<(), eyre::Report> {
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

fn print_response(side_response: &Option<SideResponse>) {
    if let Some(SideResponse { side, response }) = side_response {
        match response {
            Response::Log(log) => println!("{:?}: '{}'", side, log),
            Response::Position(position) => println!("{:?} at {}", side, position),
            Response::BatteryReadings {
                battery_voltage,
                backup_battery_voltage,
                motor_current,
            } => println!(
                "{:?} battery voltage: {} mV, backup: {} mV, current {} mV",
                side, battery_voltage, backup_battery_voltage, motor_current
            ),
            Response::ChargeState { charger_connected } => println!(
                "{:?} {}",
                side,
                if *charger_connected {
                    "charger connected"
                } else {
                    "charger not connected"
                }
            ),
            Response::PowerOff => println!("{:?} powering off", side),
        }
    }
}

fn read_port(
    port: &mut Box<dyn SerialPort>,
    buffer: &mut SliceDeque<u8>,
) -> Result<Option<SideResponse>, eyre::Report> {
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
