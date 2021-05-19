use eyre::{Report, WrapErr};
use gilrs::{Axis, Button, Event, EventType, Gilrs};
use log::error;
use messages::client::{Hoverkite, MIN_TIME_BETWEEN_TARGET_UPDATES};
use messages::{Command, Note, Response, Side, SideResponse, SpeedLimits};
use std::env;
use std::num::NonZeroU32;
use std::process::exit;
use std::thread;
use std::time::Duration;

const BAUD_RATE: u32 = 115_200;
const SLEEP_DURATION: Duration = Duration::from_millis(2);

const DEFAULT_SCALE: f32 = 50.0;
const MAX_SCALE: f32 = 100.0;

const DEFAULT_MAX_SPEED: SpeedLimits = SpeedLimits {
    negative: -200,
    positive: 30,
};
const MAX_MAX_SPEED: i16 = 300;
const MAX_SPEED_STEP: i16 = 10;

const DEFAULT_SPRING_CONSTANT: u16 = 10;
const MAX_SPRING_CONSTANT: u16 = 50;
const SPRING_CONSTANT_STEP: u16 = 2;

const CENTRE_STEP: i64 = 20;

fn main() -> Result<(), Report> {
    stable_eyre::install()?;
    pretty_env_logger::init();
    color_backtrace::install();

    let mut args = env::args();
    let binary_name = args
        .next()
        .ok_or_else(|| eyre::eyre!("Binary name missing"))?;
    if !(1..=2).contains(&args.len()) {
        eprintln!("Usage:");
        eprintln!("  {} <right serial port> [<left serial port>]", binary_name);
        exit(1);
    }
    let right_port_name = args.next().unwrap();
    let left_port_name = args.next();

    let right_port = serialport::new(&right_port_name, BAUD_RATE)
        .open()
        .map_err(|e| {
            error!(
                "Failed to open right serial port {}: {}",
                right_port_name, e
            )
        })
        .ok();
    let left_port = left_port_name.and_then(|name| {
        serialport::new(&name, BAUD_RATE)
            .open()
            .map_err(|e| error!("Failed to open left serial port {}: {}", name, e))
            .ok()
    });

    let gilrs = Gilrs::new().unwrap();

    let mut hoverkite = Hoverkite::new(right_port, left_port);

    hoverkite.play_notes(&[
        Note {
            frequency: NonZeroU32::new(100),
            duration_ms: 1000,
        },
        Note {
            frequency: NonZeroU32::new(200),
            duration_ms: 1000,
        },
        Note {
            frequency: NonZeroU32::new(300),
            duration_ms: 1000,
        },
        Note {
            frequency: NonZeroU32::new(100),
            duration_ms: 1000,
        },
        Note {
            frequency: NonZeroU32::new(200),
            duration_ms: 1000,
        },
        Note {
            frequency: NonZeroU32::new(300),
            duration_ms: 1000,
        },
    ])?;

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
    max_speed: SpeedLimits,
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
            for response in self.hoverkite.poll()? {
                print_response(&response);
            }

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
                if -self.max_speed.negative < MAX_MAX_SPEED {
                    self.max_speed.negative -= MAX_SPEED_STEP;
                    self.send_max_speed()?;
                }
            }
            EventType::ButtonPressed(Button::DPadDown, _code) => {
                if -self.max_speed.negative > MAX_SPEED_STEP {
                    self.max_speed.negative += MAX_SPEED_STEP;
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
                self.hoverkite.send_command(Side::Left, Command::Recenter)?;
            }
            EventType::ButtonPressed(Button::RightThumb, _code) => {
                self.centre_right = 0;
                self.hoverkite
                    .send_command(Side::Right, Command::Recenter)?;
            }
            EventType::ButtonPressed(Button::South, _code) => {
                self.hoverkite
                    .send_command(Side::Left, Command::ReportBattery)?;
                self.hoverkite
                    .send_command(Side::Right, Command::ReportBattery)?;
            }
            EventType::ButtonPressed(Button::East, _code) => {
                self.hoverkite
                    .send_command(Side::Left, Command::RemoveTarget)?;
                self.hoverkite
                    .send_command(Side::Right, Command::RemoveTarget)?;
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
                self.hoverkite.send_command(Side::Left, Command::PowerOff)?;
                self.hoverkite
                    .send_command(Side::Right, Command::PowerOff)?;
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
        self.hoverkite
            .set_max_speed(Side::Left, self.max_speed.invert())?;
        self.hoverkite.set_max_speed(Side::Right, self.max_speed)?;
        Ok(())
    }

    fn send_spring_constant(&mut self) -> Result<(), Report> {
        self.hoverkite
            .set_spring_constant(self.spring_constant)
            .wrap_err("Failed to set spring constant")
    }

    fn send_target(&mut self, side: Side) -> Result<(), Report> {
        let target = match side {
            Side::Left => -self.centre_left - self.offset_left,
            Side::Right => self.centre_right + self.offset_right,
        };
        self.hoverkite
            .set_target(side, target)
            .wrap_err("Failed to set target")
    }
}

fn print_response(side_response: &SideResponse) {
    match side_response.response {
        Response::Log(log) => println!("{:?}: '{}'", side_response.side, log),
        Response::Position(position) => println!("{:?} at {}", side_response.side, position),
        Response::BatteryReadings {
            battery_voltage,
            backup_battery_voltage,
            motor_current,
        } => println!(
            "{:?} battery voltage: {} mV, backup: {} mV, current {} mV",
            side_response.side, battery_voltage, backup_battery_voltage, motor_current
        ),
        Response::ChargeState { charger_connected } => println!(
            "{:?} {}",
            side_response.side,
            if charger_connected {
                "charger connected"
            } else {
                "charger not connected"
            }
        ),
        Response::PowerOff => println!("{:?} powering off", side_response.side),
    }
}
