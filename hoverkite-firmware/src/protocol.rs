use crate::buffered_tx::BufferedSerialWriter;
use crate::hoverboard::Hoverboard;
use crate::poweroff;
use core::{
    convert::TryInto,
    fmt::Debug,
    ops::{Deref, RangeInclusive},
};
use embedded_hal::blocking::serial::Write;
use embedded_hal::serial::Read;
use gd32f1x0_hal::{
    pac::{self, usart0},
    prelude::*,
    serial::{Rx, Tx},
};
use hoverkite_protocol::{Command, Message, MessageReader, ParseError};

#[macro_export]
macro_rules! log {
    ($dst:expr, $($arg:tt)*) => (
		{
			core::fmt::Write::write_char(&mut $dst, '"').unwrap();
			core::fmt::Write::write_fmt(&mut $dst, format_args!($($arg)*)).unwrap();
			core::fmt::Write::write_char(&mut $dst,'\n').unwrap();
		}
    );
}

pub fn send_position<W: Write<u8>>(serial: &mut W, position: i64, from_other_side: bool)
where
    W::Error: Debug,
{
    serial
        .bwrite_all(if from_other_side { b"i" } else { b"I" })
        .unwrap();
    serial.bwrite_all(&position.to_le_bytes()).unwrap();
}

pub fn send_secondary_log<W: Write<u8>>(serial: &mut W, log: &[u8])
where
    W::Error: Debug,
{
    serial.bwrite_all(b"'").unwrap();
    serial.bwrite_all(log).unwrap();
    serial.bwrite_all(b"\n").unwrap();
}

/// Process the given response from the secondary board.
#[cfg(feature = "primary")]
pub fn process_response(response: &[u8], hoverboard: &mut Hoverboard) -> bool {
    if response.len() < 1 {
        return false;
    }

    match response[0] {
        b'"' => {
            if response.last() != Some(&b'\n') {
                return false;
            }
            let log = &response[1..response.len() - 1];
            send_secondary_log(hoverboard.response_tx(), log);
        }
        b'I' => {
            if response.len() < 9 {
                return false;
            }
            let position = i64::from_le_bytes(response[1..9].try_into().unwrap());
            send_position(hoverboard.response_tx(), position, true);
        }
        b'p' => {
            poweroff(hoverboard);
        }
        _ => log!(
            hoverboard.response_tx(),
            "Unrecognised response {}",
            response[0]
        ),
    }
    true
}

#[cfg(feature = "primary")]
fn forward_command(hoverboard: &mut Hoverboard, command: &Command) {
    command.write_to(&mut hoverboard.serial_writer).unwrap();
}

#[cfg(feature = "secondary")]
fn forward_command(hoverboard: &mut Hoverboard, _command: &Command) {
    log!(hoverboard.response_tx(), "Secondary can't forward.");
}

/// Process the given command, returning true if a command was successfully parsed or false if not
/// enough was read yet.
pub fn process_command(
    message: Message,
    hoverboard: &mut Hoverboard,
    speed_limits: &mut RangeInclusive<i16>,
    target_position: &mut Option<i64>,
    spring_constant: &mut i64,
) -> bool {
    match message {
        Message::SecondaryCommand(sc) => {
            forward_command(hoverboard, &sc.0);
        }
        Message::Command(c) => match c {
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
                    log!(hoverboard.response_tx(), "green on");
                    hoverboard.leds.red.set_high().unwrap()
                } else {
                    log!(hoverboard.response_tx(), "green off");
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
            Command::ReportBattery => {
                let readings = hoverboard.adc_readings();
                log!(
                    hoverboard.response_tx(),
                    "Battery voltage: {} mV, backup: {} mV, current {} mV",
                    readings.battery_voltage,
                    readings.backup_battery_voltage,
                    readings.motor_current
                );
            }
            Command::ReportCharger => {
                if hoverboard.charge_state.is_low().unwrap() {
                    log!(hoverboard.response_tx(), "Charger connected");
                } else {
                    log!(hoverboard.response_tx(), "Charger not connected");
                }
            }
            Command::SetMaxSpeed(limits) => {
                log!(hoverboard.response_tx(), "max speed {:?}", limits);
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
        },
    }
    true
}

pub trait HoverboardExt {
    type CommandUsart: Deref<Target = usart0::RegisterBlock>;

    fn command_rx(&mut self) -> &mut MessageReader<Rx<Self::CommandUsart>>;
    fn response_tx(&mut self) -> &mut BufferedSerialWriter<Tx<Self::CommandUsart>>;
}

#[cfg(feature = "primary")]
impl HoverboardExt for Hoverboard {
    type CommandUsart = pac::USART0;

    fn command_rx(&mut self) -> &mut MessageReader<Rx<pac::USART0>> {
        &mut self.serial_remote_rx
    }

    fn response_tx(&mut self) -> &mut BufferedSerialWriter<Tx<pac::USART0>> {
        &mut self.serial_remote_writer
    }
}

#[cfg(feature = "secondary")]
impl HoverboardExt for Hoverboard {
    type CommandUsart = pac::USART1;

    fn command_rx(&mut self) -> &mut MessageReader<Rx<pac::USART1>> {
        &mut self.serial_rx
    }

    fn response_tx(&mut self) -> &mut BufferedSerialWriter<Tx<pac::USART1>> {
        &mut self.serial_writer
    }
}
