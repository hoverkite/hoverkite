use eyre::Report;
use gilrs::{Axis, Button, Event, EventType, Gilrs};
use log::{error, trace};
use serialport::SerialPort;
use std::thread;
use std::time::{Duration, Instant};

const LEFT_PORT: &str = "/dev/ttyUSB0";
const RIGHT_PORT: &str = "/dev/ttyUSB1";
const BAUD_RATE: u32 = 115_200;
const MIN_TIME_BETWEEN_TARGET_UPDATES: Duration = Duration::from_millis(100);
const SLEEP_DURATION: Duration = Duration::from_millis(2);

const DEFAULT_SCALE: f32 = 30.0;
const MAX_SCALE: f32 = 100.0;

const DEFAULT_MAX_SPEED: i16 = 200;
const MAX_MAX_SPEED: i16 = 300;
const MAX_SPEED_STEP: i16 = 10;

const DEFAULT_SPRING_CONSTANT: u16 = 10;
const MAX_SPRING_CONSTANT: u16 = 50;
const SPRING_CONSTANT_STEP: u16 = 2;

const CENTRE_STEP: i64 = 10;

fn main() -> Result<(), Report> {
    stable_eyre::install()?;
    pretty_env_logger::init();
    color_backtrace::install();

    let left_port = serialport::new(LEFT_PORT, BAUD_RATE)
        .open()
        .map_err(|e| error!("Failed to open left serial port {}: {}", LEFT_PORT, e))
        .ok();
    let right_port = serialport::new(RIGHT_PORT, BAUD_RATE)
        .open()
        .map_err(|e| error!("Failed to open right serial port {}: {}", RIGHT_PORT, e))
        .ok();

    let gilrs = Gilrs::new().unwrap();

    let mut controller = Controller::new(left_port, right_port, gilrs);
    controller.run()
}

struct Controller {
    left_port: Option<Box<dyn SerialPort>>,
    right_port: Option<Box<dyn SerialPort>>,
    gilrs: Gilrs,
    offset_left: i64,
    offset_right: i64,
    centre_left: i64,
    centre_right: i64,
    scale: f32,
    max_speed: i16,
    spring_constant: u16,
    /// The time that the last command was sent to the left port.
    left_last_command_time: Instant,
    /// The time that the last command was sent to the right port.
    right_last_command_time: Instant,
    /// Whether a target command still needs to be sent but wasn't because of the minimum time
    /// between updates.
    left_target_pending: bool,
    right_target_pending: bool,
}

impl Controller {
    pub fn new(
        left_port: Option<Box<dyn SerialPort>>,
        right_port: Option<Box<dyn SerialPort>>,
        gilrs: Gilrs,
    ) -> Self {
        Self {
            left_port,
            right_port,
            gilrs,
            offset_left: 0,
            offset_right: 0,
            centre_left: 0,
            centre_right: 0,
            scale: DEFAULT_SCALE,
            max_speed: DEFAULT_MAX_SPEED,
            spring_constant: DEFAULT_SPRING_CONSTANT,
            left_last_command_time: Instant::now(),
            right_last_command_time: Instant::now(),
            left_target_pending: false,
            right_target_pending: false,
        }
    }

    pub fn run(&mut self) -> Result<(), Report> {
        self.set_max_speed()?;
        self.set_spring_constant()?;

        let mut left_buffer = [0; 100];
        let mut left_length = 0;
        let mut right_buffer = [0; 100];
        let mut right_length = 0;
        loop {
            self.send_pending_targets()?;

            if let Some(Event {
                id: _,
                event,
                time: _,
            }) = self.gilrs.next_event()
            {
                self.handle_event(event)?;
            } else {
                thread::sleep(SLEEP_DURATION);
            }

            if let Some(port) = &mut self.left_port {
                Self::read_port(port, &mut left_buffer, &mut left_length, "Left")?;
            }
            if let Some(port) = &mut self.right_port {
                Self::read_port(port, &mut right_buffer, &mut right_length, "Right")?;
            }
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

    fn handle_event(&mut self, event: EventType) -> Result<(), Report> {
        match event {
            EventType::AxisChanged(Axis::LeftStickY, value, _code) => {
                self.offset_left = (self.scale * value) as i64;
                self.set_target(Side::Left)?;
            }
            EventType::AxisChanged(Axis::RightStickY, value, _code) => {
                self.offset_right = (self.scale * value) as i64;
                self.set_target(Side::Right)?;
            }
            EventType::ButtonPressed(Button::DPadLeft, _code) => {
                if self.scale > 1.0 {
                    self.scale -= 1.0;
                }
                println!("Scale {}", self.scale);
            }
            EventType::ButtonPressed(Button::DPadRight, _code) => {
                if self.scale < MAX_SCALE {
                    self.scale += 1.0;
                }
                println!("Scale {}", self.scale);
            }
            EventType::ButtonPressed(Button::DPadUp, _code) => {
                if self.max_speed < MAX_MAX_SPEED {
                    self.max_speed += MAX_SPEED_STEP;
                    self.set_max_speed()?;
                }
            }
            EventType::ButtonPressed(Button::DPadDown, _code) => {
                if self.max_speed > MAX_SPEED_STEP {
                    self.max_speed -= MAX_SPEED_STEP;
                    self.set_max_speed()?;
                }
            }
            EventType::ButtonPressed(Button::LeftTrigger, _code) => {
                self.centre_left += CENTRE_STEP;
                self.set_target(Side::Left)?;
            }
            EventType::ButtonPressed(Button::LeftTrigger2, _code) => {
                self.centre_left -= CENTRE_STEP;
                self.set_target(Side::Left)?;
            }
            EventType::ButtonPressed(Button::RightTrigger, _code) => {
                self.centre_right += CENTRE_STEP;
                self.set_target(Side::Right)?;
            }
            EventType::ButtonPressed(Button::RightTrigger2, _code) => {
                self.centre_right -= CENTRE_STEP;
                self.set_target(Side::Right)?;
            }
            EventType::ButtonPressed(Button::South, _code) => {
                self.send_command(Side::Left, &[b'b'])?;
                self.send_command(Side::Right, &[b'b'])?;
            }
            EventType::ButtonPressed(Button::West, _code) => {
                if self.spring_constant > SPRING_CONSTANT_STEP {
                    self.spring_constant -= SPRING_CONSTANT_STEP;
                    self.set_spring_constant()?;
                }
            }
            EventType::ButtonPressed(Button::North, _code) => {
                if self.spring_constant < MAX_SPRING_CONSTANT {
                    self.spring_constant += SPRING_CONSTANT_STEP;
                    self.set_spring_constant()?;
                }
            }
            EventType::ButtonPressed(Button::Mode, _code) => {
                // Power off
                self.send_command(Side::Left, &[b'p'])?;
                self.send_command(Side::Right, &[b'p'])?;
            }
            EventType::ButtonPressed(button, code) => {
                println!("Button {:?} pressed: {:?}", button, code);
            }
            _ => {}
        }
        Ok(())
    }

    fn set_max_speed(&mut self) -> Result<(), Report> {
        println!("Max speed: {}", self.max_speed);
        let mut command = vec![b'S'];
        command.extend_from_slice(&self.max_speed.to_le_bytes());
        self.send_command(Side::Left, &command)?;
        self.send_command(Side::Right, &command)?;
        Ok(())
    }

    fn set_spring_constant(&mut self) -> Result<(), Report> {
        println!("Spring constant: {}", self.spring_constant);
        let mut command = vec![b'K'];
        command.extend_from_slice(&self.spring_constant.to_le_bytes());
        self.send_command(Side::Left, &command)?;
        self.send_command(Side::Right, &command)?;
        Ok(())
    }

    fn send_pending_targets(&mut self) -> Result<(), Report> {
        let now = Instant::now();
        if self.left_target_pending
            && now > self.left_last_command_time + MIN_TIME_BETWEEN_TARGET_UPDATES
        {
            self.left_target_pending = false;
            self.set_target(Side::Left)?;
        }
        if self.right_target_pending
            && now > self.right_last_command_time + MIN_TIME_BETWEEN_TARGET_UPDATES
        {
            self.right_target_pending = false;
            self.set_target(Side::Right)?;
        }
        Ok(())
    }

    fn set_target(&mut self, side: Side) -> Result<(), Report> {
        let now = Instant::now();
        let target = match side {
            Side::Left => {
                if now < self.left_last_command_time + MIN_TIME_BETWEEN_TARGET_UPDATES {
                    self.left_target_pending = true;
                    return Ok(());
                }
                self.centre_left + self.offset_left
            }
            Side::Right => {
                if now < self.right_last_command_time + MIN_TIME_BETWEEN_TARGET_UPDATES {
                    self.right_target_pending = true;
                    return Ok(());
                }
                self.centre_right + self.offset_right
            }
        };
        println!("Target {:?} {}", side, target);
        let mut command = vec![b'T'];
        command.extend_from_slice(&target.to_le_bytes());
        self.send_command(side, &command)
    }

    fn send_command(&mut self, side: Side, command: &[u8]) -> Result<(), Report> {
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

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
enum Side {
    Left,
    Right,
}
