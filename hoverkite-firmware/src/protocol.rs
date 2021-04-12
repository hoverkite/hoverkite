use crate::buffered_tx::BufferedSerialWriter;
use crate::hoverboard::Hoverboard;
use crate::poweroff;
use core::{
    convert::TryInto,
    fmt::Debug,
    ops::{Deref, RangeInclusive},
};
use embedded_hal::blocking::serial::Write;
use gd32f1x0_hal::{
    pac::{self, usart0},
    prelude::*,
    serial::{Rx, Tx},
};

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
        _ => log!(
            hoverboard.response_tx(),
            "Unrecognised response {}",
            response[0]
        ),
    }
    true
}

fn forward_command(hoverboard: &mut Hoverboard, command: &[u8]) {
    #[cfg(feature = "primary")]
    hoverboard.serial_writer.bwrite_all(command).unwrap();
    #[cfg(feature = "secondary")]
    log!(hoverboard.response_tx(), "Secondary can't forward.");
}

/// Process the given command, returning true if a command was successfully parsed or false if not
/// enough was read yet.
pub fn process_command(
    command: &[u8],
    hoverboard: &mut Hoverboard,
    speed_limits: &mut RangeInclusive<i16>,
    target_position: &mut Option<i64>,
    spring_constant: &mut i64,
) -> bool {
    if command.len() < 1 {
        return false;
    }

    match command[0] {
        b'F' => {
            if command.len() < 2 {
                return false;
            }
            let forward_length = command[1];
            if command.len() < forward_length as usize + 2 {
                return false;
            }
            forward_command(hoverboard, &command[2..]);
        }
        b'l' => {
            if command.len() < 2 {
                return false;
            }
            match command[1] {
                b'0' => {
                    log!(hoverboard.response_tx(), "LED off");
                    hoverboard.leds.side.set_low().unwrap()
                }
                b'1' => {
                    log!(hoverboard.response_tx(), "LED on");
                    hoverboard.leds.side.set_high().unwrap()
                }
                _ => log!(hoverboard.response_tx(), "LED unrecognised"),
            }
        }
        b'o' => {
            if command.len() < 2 {
                return false;
            }
            match command[1] {
                b'0' => {
                    log!(hoverboard.response_tx(), "orange off");
                    hoverboard.leds.orange.set_low().unwrap()
                }
                b'1' => {
                    log!(hoverboard.response_tx(), "orange on");
                    hoverboard.leds.orange.set_high().unwrap()
                }
                _ => log!(hoverboard.response_tx(), "LED unrecognised"),
            }
        }
        b'r' => {
            if command.len() < 2 {
                return false;
            }
            match command[1] {
                b'0' => {
                    log!(hoverboard.response_tx(), "red off");
                    hoverboard.leds.red.set_low().unwrap()
                }
                b'1' => {
                    log!(hoverboard.response_tx(), "red on");
                    hoverboard.leds.red.set_high().unwrap()
                }
                _ => log!(hoverboard.response_tx(), "LED unrecognised"),
            }
        }
        b'g' => {
            if command.len() < 2 {
                return false;
            }
            match command[1] {
                b'0' => {
                    log!(hoverboard.response_tx(), "green off");
                    hoverboard.leds.green.set_low().unwrap()
                }
                b'1' => {
                    log!(hoverboard.response_tx(), "green on");
                    hoverboard.leds.green.set_high().unwrap()
                }
                _ => log!(hoverboard.response_tx(), "LED unrecognised"),
            }
        }
        b'b' => {
            let readings = hoverboard.adc_readings();
            log!(
                hoverboard.response_tx(),
                "Battery voltage: {} mV, backup: {} mV, current {} mV",
                readings.battery_voltage,
                readings.backup_battery_voltage,
                readings.motor_current
            );
        }
        b'c' => {
            if hoverboard.charge_state.is_low().unwrap() {
                log!(hoverboard.response_tx(), "Charger connected");
            } else {
                log!(hoverboard.response_tx(), "Charger not connected");
            }
        }
        b'S' => {
            if command.len() < 5 {
                return false;
            }
            let min_power = i16::from_le_bytes(command[1..3].try_into().unwrap());
            let max_power = i16::from_le_bytes(command[3..5].try_into().unwrap());
            log!(
                hoverboard.response_tx(),
                "max speed {}..{}",
                min_power,
                max_power
            );
            *speed_limits = min_power..=max_power;
        }
        b'K' => {
            if command.len() < 3 {
                return false;
            }
            let spring = u16::from_le_bytes(command[1..3].try_into().unwrap()).into();
            log!(hoverboard.response_tx(), "Spring constant {}", spring);
            *spring_constant = spring;
        }
        b'n' => {
            log!(hoverboard.response_tx(), "No target position");
            *target_position = None;
        }
        b'T' => {
            if command.len() < 9 {
                return false;
            }
            let target = i64::from_le_bytes(command[1..9].try_into().unwrap());
            log!(hoverboard.response_tx(), "Target position {}", target);
            *target_position = Some(target);
        }
        b'e' => {
            log!(hoverboard.response_tx(), "Recentre");
            hoverboard.recentre_motor();
            *target_position = Some(0);
        }
        b'+' => {
            let target = target_position.unwrap_or(0) + 10;
            log!(hoverboard.response_tx(), "Target position {}", target);
            *target_position = Some(target);
        }
        b'-' => {
            let target = target_position.unwrap_or(0) - 10;
            log!(hoverboard.response_tx(), "Target position {}", target);
            *target_position = Some(target);
        }
        b'p' => poweroff(hoverboard),
        _ => log!(
            hoverboard.response_tx(),
            "Unrecognised command {}",
            command[0]
        ),
    }
    true
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
