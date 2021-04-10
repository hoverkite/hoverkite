mod hoverkite;

use eyre::Report;
use gilrs::{Axis, Button, Event, EventType, Gilrs};
use hoverkite::{Hoverkite, Side, MIN_TIME_BETWEEN_TARGET_UPDATES};
use log::error;
use std::env;
use std::ops::RangeInclusive;
use std::process::exit;
use std::thread;
use std::time::Duration;

const BAUD_RATE: u32 = 115_200;
const SLEEP_DURATION: Duration = Duration::from_millis(2);

const DEFAULT_SCALE: f32 = 30.0;
const MAX_SCALE: f32 = 100.0;

const DEFAULT_MAX_SPEED: RangeInclusive<i16> = -200..=30;
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

    let mut args = env::args();
    let binary_name = args
        .next()
        .ok_or_else(|| eyre::eyre!("Binary name missing"))?;
    if args.len() != 2 {
        eprintln!("Usage:");
        eprintln!("  {} <left serial port> <right serial port>", binary_name);
        exit(1);
    }
    let left_port_name = args.next().unwrap();
    let right_port_name = args.next().unwrap();

    let left_port = serialport::new(&left_port_name, BAUD_RATE)
        .open()
        .map_err(|e| error!("Failed to open left serial port {}: {}", left_port_name, e))
        .ok();
    let right_port = serialport::new(&right_port_name, BAUD_RATE)
        .open()
        .map_err(|e| {
            error!(
                "Failed to open right serial port {}: {}",
                right_port_name, e
            )
        })
        .ok();

    let gilrs = Gilrs::new().unwrap();

    let hoverkite = Hoverkite::new(left_port, right_port);
    let mut controller = Controller::new(hoverkite, gilrs);
    controller.run()
}

struct Controller {
    hoverkite: Hoverkite,
    gilrs: Gilrs,
    offset_left: i64,
    offset_right: i64,
    centre_left: i64,
    centre_right: i64,
    scale: f32,
    max_speed: RangeInclusive<i16>,
    spring_constant: u16,
}

impl Controller {
    pub fn new(hoverkite: Hoverkite, gilrs: Gilrs) -> Self {
        Self {
            hoverkite,
            gilrs,
            offset_left: 0,
            offset_right: 0,
            centre_left: 0,
            centre_right: 0,
            scale: DEFAULT_SCALE,
            max_speed: DEFAULT_MAX_SPEED,
            spring_constant: DEFAULT_SPRING_CONSTANT,
        }
    }

    pub fn run(&mut self) -> Result<(), Report> {
        self.send_max_speed()?;
        thread::sleep(MIN_TIME_BETWEEN_TARGET_UPDATES);
        self.send_spring_constant()?;

        loop {
            self.hoverkite.process()?;

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
        }
    }

    fn handle_event(&mut self, event: EventType) -> Result<(), Report> {
        match event {
            EventType::AxisChanged(Axis::LeftStickY, value, _code) => {
                self.offset_left = (self.scale * value) as i64;
                self.send_target(Side::Left)?;
            }
            EventType::AxisChanged(Axis::RightStickY, value, _code) => {
                self.offset_right = (self.scale * value) as i64;
                self.send_target(Side::Right)?;
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
                if -self.max_speed.start() < MAX_MAX_SPEED {
                    self.max_speed =
                        self.max_speed.start() - MAX_SPEED_STEP..=*self.max_speed.end();
                    self.send_max_speed()?;
                }
            }
            EventType::ButtonPressed(Button::DPadDown, _code) => {
                if -self.max_speed.start() > MAX_SPEED_STEP {
                    self.max_speed =
                        self.max_speed.start() + MAX_SPEED_STEP..=*self.max_speed.end();
                    self.send_max_speed()?;
                }
            }
            EventType::ButtonPressed(Button::LeftTrigger, _code) => {
                self.centre_left += CENTRE_STEP;
                self.send_target(Side::Left)?;
            }
            EventType::ButtonPressed(Button::LeftTrigger2, _code) => {
                self.centre_left -= CENTRE_STEP;
                self.send_target(Side::Left)?;
            }
            EventType::ButtonPressed(Button::RightTrigger, _code) => {
                self.centre_right += CENTRE_STEP;
                self.send_target(Side::Right)?;
            }
            EventType::ButtonPressed(Button::RightTrigger2, _code) => {
                self.centre_right -= CENTRE_STEP;
                self.send_target(Side::Right)?;
            }
            EventType::ButtonPressed(Button::LeftThumb, _code) => {
                self.centre_left = 0;
                self.hoverkite.send_command(Side::Left, &[b'e'])?;
            }
            EventType::ButtonPressed(Button::RightThumb, _code) => {
                self.centre_right = 0;
                self.hoverkite.send_command(Side::Right, &[b'e'])?;
            }
            EventType::ButtonPressed(Button::South, _code) => {
                self.hoverkite.send_command(Side::Left, &[b'b'])?;
                self.hoverkite.send_command(Side::Right, &[b'b'])?;
            }
            EventType::ButtonPressed(Button::East, _code) => {
                self.hoverkite.send_command(Side::Left, &[b'n'])?;
                self.hoverkite.send_command(Side::Right, &[b'n'])?;
            }
            EventType::ButtonPressed(Button::West, _code) => {
                if self.spring_constant > SPRING_CONSTANT_STEP {
                    self.spring_constant -= SPRING_CONSTANT_STEP;
                    self.send_spring_constant()?;
                }
            }
            EventType::ButtonPressed(Button::North, _code) => {
                if self.spring_constant < MAX_SPRING_CONSTANT {
                    self.spring_constant += SPRING_CONSTANT_STEP;
                    self.send_spring_constant()?;
                }
            }
            EventType::ButtonPressed(Button::Mode, _code) => {
                // Power off
                self.hoverkite.send_command(Side::Left, &[b'p'])?;
                self.hoverkite.send_command(Side::Right, &[b'p'])?;
            }
            EventType::ButtonPressed(button, code) => {
                println!("Button {:?} pressed: {:?}", button, code);
            }
            _ => {}
        }
        Ok(())
    }

    pub fn send_max_speed(&mut self) -> Result<(), Report> {
        // Invert left
        self.hoverkite.set_max_speed(
            Side::Left,
            &(-self.max_speed.end()..=-self.max_speed.start()),
        )?;
        self.hoverkite.set_max_speed(Side::Right, &self.max_speed)?;
        Ok(())
    }

    fn send_spring_constant(&mut self) -> Result<(), Report> {
        self.hoverkite.set_spring_constant(self.spring_constant)
    }

    fn send_target(&mut self, side: Side) -> Result<(), Report> {
        let target = match side {
            Side::Left => -self.centre_left - self.offset_left,
            Side::Right => self.centre_right + self.offset_right,
        };
        self.hoverkite.set_target(side, target)
    }
}
