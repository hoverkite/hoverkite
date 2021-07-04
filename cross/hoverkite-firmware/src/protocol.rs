use crate::buffered_tx::BufferedSerialWriter;
use crate::circular_buffer::CircularBuffer;
use crate::hoverboard::Hoverboard;
use crate::poweroff;
use core::{fmt::Debug, ops::Deref};
use embedded_hal::blocking::serial::Write;
use gd32f1x0_hal::{
    pac::{self, usart0},
    prelude::*,
    serial::{Rx, Tx},
};
#[allow(unused_imports)]
use messages::{
    Command, DirectedCommand, Note, ProtocolError, Response, Side, SideResponse, SpeedLimits,
};
#[allow(unused_imports)]
use nb::Error::{Other, WouldBlock};

#[macro_export]
macro_rules! log {
    ($dst:expr, $($arg:tt)*) => (
		{
            ::messages::SideResponse {
                side: crate::protocol::THIS_SIDE,
                response: ::messages::Response::log_from_fmt(format_args!($($arg)*))
            }.write_to($dst).unwrap()
		}
    );
}

#[cfg(feature = "primary")]
pub const THIS_SIDE: Side = Side::Right;
#[cfg(feature = "secondary")]
pub const THIS_SIDE: Side = Side::Left;

pub fn send_position<W: Write<u8>>(serial: &mut W, position: i64)
where
    W::Error: Debug,
{
    SideResponse {
        side: THIS_SIDE,
        response: Response::Position(position),
    }
    .write_to(serial)
    .unwrap();
}

fn send_battery_readings<W: Write<u8>>(
    serial: &mut W,
    battery_voltage: u16,
    backup_battery_voltage: u16,
    motor_current: u16,
) where
    W::Error: Debug,
{
    SideResponse {
        side: THIS_SIDE,
        response: Response::BatteryReadings {
            battery_voltage,
            backup_battery_voltage,
            motor_current,
        },
    }
    .write_to(serial)
    .unwrap();
}

fn send_charge_state<W: Write<u8>>(serial: &mut W, charger_connected: bool)
where
    W::Error: Debug,
{
    SideResponse {
        side: THIS_SIDE,
        response: Response::ChargeState { charger_connected },
    }
    .write_to(serial)
    .unwrap();
}

/// Process the given response from the secondary board.
#[cfg(feature = "primary")]
pub fn process_response(response: &[u8], hoverboard: &mut Hoverboard) -> bool {
    match SideResponse::parse_exact(response) {
        Ok(side_response) => {
            side_response.write_to(hoverboard.response_tx()).unwrap();
            if side_response.response == Response::PowerOff {
                poweroff(hoverboard);
            }
            true
        }
        Err(WouldBlock) => false,
        Err(Other(protocol_error)) => {
            log!(
                hoverboard.response_tx(),
                "Unrecognised response {}",
                protocol_error
            );
            true
        }
    }
}

#[cfg(feature = "primary")]
fn forward_command(hoverboard: &mut Hoverboard, command: &DirectedCommand) {
    command.write_to(&mut hoverboard.serial_writer).unwrap();
}

#[cfg(feature = "secondary")]
fn forward_command(hoverboard: &mut Hoverboard, _command: &DirectedCommand) {
    log!(hoverboard.response_tx(), "Secondary can't forward.");
}

/// Process the given command, returning true if a command was successfully parsed or false if not
/// enough was read yet.
pub fn process_command<const L: usize>(
    command: &[u8],
    hoverboard: &mut Hoverboard,
    speed_limits: &mut SpeedLimits,
    target_position: &mut Option<i64>,
    spring_constant: &mut i64,
    note_queue: &mut CircularBuffer<Note, L>,
) -> bool {
    let message = match DirectedCommand::parse(command) {
        Ok(message) => message,
        Err(nb::Error::WouldBlock) => return false,
        Err(err) => {
            log!(
                hoverboard.response_tx(),
                "Unrecognised command {} or problem {:?}",
                command[0],
                err
            );
            // Make sure the buffer progresses here, and we don't get stuck with the same duff
            // input bytes at the start of our buffer forever.
            return true;
        }
    };

    if message.side == THIS_SIDE {
        handle_command(
            message.command,
            hoverboard,
            speed_limits,
            target_position,
            spring_constant,
            note_queue,
        );
    } else {
        forward_command(hoverboard, &message);
    }
    true
}

pub fn handle_command<const L: usize>(
    command: Command,
    hoverboard: &mut Hoverboard,
    speed_limits: &mut SpeedLimits,
    target_position: &mut Option<i64>,
    spring_constant: &mut i64,
    note_queue: &mut CircularBuffer<Note, L>,
) {
    match command {
        Command::SetSideLed(on) => {
            if on {
                log!(hoverboard.response_tx(), "side LED on");
                hoverboard.leds.side.set_high().unwrap()
            } else {
                log!(hoverboard.response_tx(), "side LED off");
                hoverboard.leds.side.set_low().unwrap()
            }
        }
        Command::SetOrangeLed(on) => {
            if on {
                log!(hoverboard.response_tx(), "orange on");
                hoverboard.leds.orange.set_high().unwrap()
            } else {
                log!(hoverboard.response_tx(), "orange off");
                hoverboard.leds.orange.set_low().unwrap()
            }
        }
        Command::SetRedLed(on) => {
            if on {
                log!(hoverboard.response_tx(), "red on");
                hoverboard.leds.red.set_high().unwrap()
            } else {
                log!(hoverboard.response_tx(), "red off");
                hoverboard.leds.red.set_low().unwrap()
            }
        }
        Command::SetGreenLed(on) => {
            if on {
                log!(hoverboard.response_tx(), "green on");
                hoverboard.leds.green.set_high().unwrap()
            } else {
                log!(hoverboard.response_tx(), "green off");
                hoverboard.leds.green.set_low().unwrap()
            }
        }
        Command::AddBuzzerNote(note) => {
            if !note_queue.add(note) {
                log!(
                    hoverboard.response_tx(),
                    "Note queue full, dropping {}",
                    note
                );
            }
        }
        Command::ReportBattery => {
            let readings = hoverboard.adc_readings();
            send_battery_readings(
                hoverboard.response_tx(),
                readings.battery_voltage,
                readings.backup_battery_voltage,
                readings.motor_current,
            );
        }
        Command::ReportCharger => {
            let charger_connected = hoverboard.charge_state.is_low().unwrap();
            send_charge_state(hoverboard.response_tx(), charger_connected);
        }
        Command::SetMaxSpeed(limits) => {
            log!(hoverboard.response_tx(), "Max speed {:?}", limits);
            *speed_limits = limits;
        }
        Command::SetSpringConstant(spring) => {
            log!(hoverboard.response_tx(), "Spring constant {}", spring);
            *spring_constant = spring as i64;
        }
        Command::RemoveTarget => {
            log!(hoverboard.response_tx(), "No target position");
            *target_position = None;
        }
        Command::SetTarget(target) => {
            *target_position = Some(target);
        }
        Command::Recenter => {
            log!(hoverboard.response_tx(), "Recenter");
            hoverboard.recenter_motor();
            *target_position = Some(0);
        }
        Command::IncrementTarget => {
            let target = target_position.unwrap_or(0) + 10;
            log!(hoverboard.response_tx(), "Target position {}", target);
            *target_position = Some(target);
        }
        Command::DecrementTarget => {
            let target = target_position.unwrap_or(0) - 10;
            log!(hoverboard.response_tx(), "Target position {}", target);
            *target_position = Some(target);
        }
        Command::PowerOff => poweroff(hoverboard),
        Command::TestMotor => {
            log!(hoverboard.response_tx(), "Setting motor PWM for test");
            log!(
                hoverboard.response_tx(),
                "yellow (PA8/PB13) = 0%, blue (PA9/PB14) = 25%, green (PA10/PB15) = 50%"
            );
            hoverboard.set_motor_pwm_for_test(0, 25, 50);
        }
    }
}

pub trait HoverboardExt {
    type CommandUsart: Deref<Target = usart0::RegisterBlock>;

    fn command_rx(&mut self) -> &mut Rx<Self::CommandUsart>;
    fn response_tx(&mut self) -> &mut BufferedSerialWriter<Tx<Self::CommandUsart>>;
}

#[cfg(feature = "primary")]
impl HoverboardExt for Hoverboard {
    type CommandUsart = pac::USART0;

    fn command_rx(&mut self) -> &mut Rx<pac::USART0> {
        &mut self.serial_remote_rx
    }

    fn response_tx(&mut self) -> &mut BufferedSerialWriter<Tx<pac::USART0>> {
        &mut self.serial_remote_writer
    }
}

#[cfg(feature = "secondary")]
impl HoverboardExt for Hoverboard {
    type CommandUsart = pac::USART1;

    fn command_rx(&mut self) -> &mut Rx<pac::USART1> {
        &mut self.serial_rx
    }

    fn response_tx(&mut self) -> &mut BufferedSerialWriter<Tx<pac::USART1>> {
        &mut self.serial_writer
    }
}
