use eyre::{Context, Report};
use gilrs::{Axis, Button, Event, EventType, Gilrs};
use serialport::SerialPort;
use std::thread;
use std::time::{Duration, Instant};

const LEFT_PORT: &str = "/dev/ttyUSB0";
const BAUD_RATE: u32 = 115_200;
const MIN_TIME_BETWEEN_TARGET_UPDATES: Duration = Duration::from_millis(100);
const SLEEP_DURATION: Duration = Duration::from_millis(2);
const MAX_SPRING_CONSTANT: u16 = 50;

fn main() -> Result<(), Report> {
    stable_eyre::install()?;
    pretty_env_logger::init();
    color_backtrace::install();

    let left_port = serialport::new(LEFT_PORT, BAUD_RATE)
        .open()
        .wrap_err_with(|| format!("Failed to open left serial port {}", LEFT_PORT))?;

    let gilrs = Gilrs::new().unwrap();

    let mut controller = Controller::new(left_port, gilrs);
    controller.run()
}

struct Controller {
    left_port: Box<dyn SerialPort>,
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
    right_last_command_time: Instant,
    /// Whether a target command still needs to be sent but wasn't because of the minimum time
    /// between updates.
    left_target_pending: bool,
    right_target_pending: bool,
}

impl Controller {
    pub fn new(left_port: Box<dyn SerialPort>, gilrs: Gilrs) -> Self {
        Self {
            left_port,
            gilrs,
            offset_left: 0,
            offset_right: 0,
            centre_left: 0,
            centre_right: 0,
            scale: 10.0,
            max_speed: 200,
            spring_constant: 10,
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

            if self.left_port.bytes_to_read()? > 0 {
                left_length += self.left_port.read(&mut left_buffer[left_length..])?;
                if let Some(end_of_line) =
                    left_buffer[0..left_length].iter().position(|&c| c == b'\n')
                {
                    let string = String::from_utf8_lossy(&left_buffer[0..end_of_line]);
                    println!("Left: '{}'", string);
                    let remaining_length = left_length - end_of_line - 1;
                    let remaining_bytes = left_buffer[end_of_line + 1..left_length].to_owned();
                    left_buffer[0..remaining_length].clone_from_slice(&remaining_bytes);
                    left_length = remaining_length;
                } else if left_length == left_buffer.len() {
                    let string = String::from_utf8_lossy(&left_buffer[0..left_length]);
                    println!("Left: '{}'", string);
                    left_length = 0;
                }
            }
        }
    }

    fn handle_event(&mut self, event: EventType) -> Result<(), Report> {
        let centre_step = 10;
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
                if self.scale < 100.0 {
                    self.scale += 1.0;
                }
                println!("Scale {}", self.scale);
            }
            EventType::ButtonPressed(Button::DPadUp, _code) => {
                if self.max_speed < 300 {
                    self.max_speed += 10;
                    self.set_max_speed()?;
                }
            }
            EventType::ButtonPressed(Button::DPadDown, _code) => {
                if self.max_speed > 10 {
                    self.max_speed -= 10;
                    self.set_max_speed()?;
                }
            }
            EventType::ButtonPressed(Button::LeftTrigger, _code) => {
                self.centre_left += centre_step;
                self.set_target(Side::Left)?;
            }
            EventType::ButtonPressed(Button::LeftTrigger2, _code) => {
                self.centre_left -= centre_step;
                self.set_target(Side::Left)?;
            }
            EventType::ButtonPressed(Button::RightTrigger, _code) => {
                self.centre_right += centre_step;
                self.set_target(Side::Right)?;
            }
            EventType::ButtonPressed(Button::RightTrigger2, _code) => {
                self.centre_right -= centre_step;
                self.set_target(Side::Right)?;
            }
            EventType::ButtonPressed(Button::South, _code) => {
                self.send_command(Side::Left, &[b'b'])?;
                self.send_command(Side::Right, &[b'b'])?;
            }
            EventType::ButtonPressed(Button::West, _code) => {
                if self.spring_constant > 2 {
                    self.spring_constant -= 2;
                    self.set_spring_constant()?;
                }
            }
            EventType::ButtonPressed(Button::North, _code) => {
                if self.spring_constant < MAX_SPRING_CONSTANT {
                    self.spring_constant += 2;
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
            Side::Right => return Ok(()),
        };
        log::trace!("Sending command: {:?}", command);
        port.write_all(command)?;
        Ok(())
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
enum Side {
    Left,
    Right,
}
