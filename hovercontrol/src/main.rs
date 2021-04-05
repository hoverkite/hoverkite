use eyre::{Context, Report};
use gilrs::{Axis, Button, Event, EventType, Gilrs};
use serialport::SerialPort;

const LEFT_PORT: &str = "/dev/ttyUSB0";
const BAUD_RATE: u32 = 115_200;

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
        }
    }

    pub fn run(&mut self) -> Result<(), Report> {
        loop {
            while let Some(Event {
                id: _,
                event,
                time: _,
            }) = self.gilrs.next_event()
            {
                self.handle_event(event)?;
            }
        }
    }

    fn handle_event(&mut self, event: EventType) -> Result<(), Report> {
        let centre_step = 10;
        match event {
            EventType::AxisChanged(Axis::LeftStickY, value, _code) => {
                self.offset_left = (self.scale * value) as i64;
                self.set_target(Side::Left, self.centre_left + self.offset_left)?;
            }
            EventType::AxisChanged(Axis::RightStickY, value, _code) => {
                self.offset_right = (self.scale * value) as i64;
                self.set_target(Side::Right, self.centre_right + self.offset_right)?;
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
            EventType::ButtonPressed(Button::LeftTrigger, _code) => {
                self.centre_left += centre_step;
                self.set_target(Side::Left, self.centre_left + self.offset_left)?;
            }
            EventType::ButtonPressed(Button::LeftTrigger2, _code) => {
                self.centre_left -= centre_step;
                self.set_target(Side::Left, self.centre_left + self.offset_left)?;
            }
            EventType::ButtonPressed(Button::RightTrigger, _code) => {
                self.centre_right += centre_step;
                self.set_target(Side::Right, self.centre_right + self.offset_right)?;
            }
            EventType::ButtonPressed(Button::RightTrigger2, _code) => {
                self.centre_right -= centre_step;
                self.set_target(Side::Right, self.centre_right + self.offset_right)?;
            }
            EventType::ButtonPressed(button, code) => {
                println!("Button {:?} pressed: {:?}", button, code);
            }
            _ => {}
        }
        Ok(())
    }

    fn set_target(&mut self, side: Side, target: i64) -> Result<(), Report> {
        println!("Target {:?} {}", side, target);
        let mut command = vec![b'T'];
        command.extend_from_slice(&target.to_le_bytes());
        self.send_command(side, &command)
    }

    fn send_command(&mut self, side: Side, command: &[u8]) -> Result<(), Report> {
        let port = match side {
            Side::Left => &mut self.left_port,
            Side::Right => return Ok(()),
        };
        port.write_all(command)?;
        Ok(())
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
enum Side {
    Left,
    Right,
}
