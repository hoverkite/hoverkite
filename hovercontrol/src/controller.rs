use crate::homie::Homie;
use eyre::{Report, WrapErr};
use gilrs::{Axis, Button, Event, EventType, Gilrs};
use messages::client::{Hoverkite, MIN_TIME_BETWEEN_TARGET_UPDATES};
use messages::{Command, Response, Side, SideResponse, SpeedLimits};
use std::thread;
use std::time::Duration;

const SLEEP_DURATION: Duration = Duration::from_millis(2);

pub const DEFAULT_SCALE: f32 = 50.0;
pub const MAX_SCALE: f32 = 100.0;

pub const DEFAULT_MAX_SPEED: SpeedLimits = SpeedLimits {
    negative: -200,
    positive: 30,
};
pub const MAX_MAX_SPEED: i16 = 300;
const MAX_SPEED_STEP: i16 = 10;

pub const DEFAULT_SPRING_CONSTANT: u16 = 10;
pub const MAX_SPRING_CONSTANT: u16 = 50;
pub const SPRING_CONSTANT_STEP: u16 = 2;

const CENTRE_STEP: i64 = 20;

pub struct Controller {
    hoverkite: Hoverkite,
    gilrs: Gilrs,
    homie: Homie,
    offset_left: i64,
    offset_right: i64,
    centre_left: i64,
    centre_right: i64,
    scale: f32,
    max_speed: SpeedLimits,
    spring_constant: u16,
}

impl Controller {
    pub fn new(hoverkite: Hoverkite, gilrs: Gilrs, homie: Homie) -> Self {
        Self {
            hoverkite,
            gilrs,
            homie,
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
                self.send_response(&response);
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

    fn send_response(&self, response: &SideResponse) {
        match response.response {
            Response::Position(position) => match response.side {
                Side::Left => self.homie.send_position(Side::Left, -position),
                Side::Right => self.homie.send_position(Side::Right, position),
            },
            Response::BatteryReadings {
                battery_voltage,
                backup_battery_voltage,
                motor_current,
            } => self.homie.send_battery_readings(
                response.side,
                battery_voltage,
                backup_battery_voltage,
                motor_current,
            ),
            Response::ChargeState { charger_connected } => {
                self.homie
                    .send_charge_state(response.side, charger_connected);
            }
            _ => {}
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
                self.homie.send_scale(self.scale);
            }
            EventType::ButtonPressed(Button::DPadRight, _code) => {
                if self.scale < MAX_SCALE {
                    self.scale += 1.0;
                }
                println!("Scale {}", self.scale);
                self.homie.send_scale(self.scale);
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
                self.homie.send_centre(Side::Left, self.centre_left);
            }
            EventType::ButtonPressed(Button::LeftTrigger2, _code) => {
                self.centre_left -= CENTRE_STEP;
                self.send_target(Side::Left)?;
                self.homie.send_centre(Side::Left, self.centre_left);
            }
            EventType::ButtonPressed(Button::RightTrigger, _code) => {
                self.centre_right += CENTRE_STEP;
                self.send_target(Side::Right)?;
                self.homie.send_centre(Side::Right, self.centre_right);
            }
            EventType::ButtonPressed(Button::RightTrigger2, _code) => {
                self.centre_right -= CENTRE_STEP;
                self.send_target(Side::Right)?;
                self.homie.send_centre(Side::Right, self.centre_right);
            }
            EventType::ButtonPressed(Button::LeftThumb, _code) => {
                self.centre_left = 0;
                self.hoverkite.send_command(Side::Left, Command::Recenter)?;
                self.homie.send_centre(Side::Left, self.centre_left);
                self.homie.send_target(Side::Left, 0);
            }
            EventType::ButtonPressed(Button::RightThumb, _code) => {
                self.centre_right = 0;
                self.hoverkite
                    .send_command(Side::Right, Command::Recenter)?;
                self.homie.send_centre(Side::Right, self.centre_right);
                self.homie.send_target(Side::Right, 0);
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

    fn send_max_speed(&mut self) -> Result<(), Report> {
        // Invert left
        self.hoverkite
            .set_max_speed(Side::Left, self.max_speed.invert())?;
        self.hoverkite.set_max_speed(Side::Right, self.max_speed)?;
        self.homie.send_max_speed(self.max_speed);
        Ok(())
    }

    fn send_spring_constant(&mut self) -> Result<(), Report> {
        self.hoverkite
            .set_spring_constant(self.spring_constant)
            .wrap_err("Failed to set spring constant")?;
        self.homie.send_spring_constant(self.spring_constant);
        Ok(())
    }

    fn send_target(&mut self, side: Side) -> Result<(), Report> {
        let target = match side {
            Side::Left => self.centre_left + self.offset_left,
            Side::Right => self.centre_right + self.offset_right,
        };
        let target_maybe_negated = match side {
            Side::Left => -target,
            Side::Right => target,
        };
        self.hoverkite
            .set_target(side, target_maybe_negated)
            .wrap_err("Failed to set target")?;
        self.homie.send_target(side, target);
        Ok(())
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
