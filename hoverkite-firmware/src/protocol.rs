use crate::hoverboard::Hoverboard;
use crate::poweroff;
use core::{convert::TryInto, fmt::Debug, ops::RangeInclusive};
use embedded_hal::blocking::serial::Write;
use gd32f1x0_hal::prelude::*;

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

pub fn send_position<W: Write<u8>>(serial: &mut W, position: i64)
where
    W::Error: Debug,
{
    serial.bwrite_all(b"P").unwrap();
    serial.bwrite_all(&position.to_le_bytes()).unwrap();
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
        b'l' => {
            if command.len() < 2 {
                return false;
            }
            match command[1] {
                b'0' => {
                    log!(hoverboard.serial_writer, "LED off");
                    hoverboard.leds.side.set_low().unwrap()
                }
                b'1' => {
                    log!(hoverboard.serial_writer, "LED on");
                    hoverboard.leds.side.set_high().unwrap()
                }
                _ => log!(hoverboard.serial_writer, "LED unrecognised"),
            }
        }
        b'o' => {
            if command.len() < 2 {
                return false;
            }
            match command[1] {
                b'0' => {
                    log!(hoverboard.serial_writer, "orange off");
                    hoverboard.leds.orange.set_low().unwrap()
                }
                b'1' => {
                    log!(hoverboard.serial_writer, "orange on");
                    hoverboard.leds.orange.set_high().unwrap()
                }
                _ => log!(hoverboard.serial_writer, "LED unrecognised"),
            }
        }
        b'r' => {
            if command.len() < 2 {
                return false;
            }
            match command[1] {
                b'0' => {
                    log!(hoverboard.serial_writer, "red off");
                    hoverboard.leds.red.set_low().unwrap()
                }
                b'1' => {
                    log!(hoverboard.serial_writer, "red on");
                    hoverboard.leds.red.set_high().unwrap()
                }
                _ => log!(hoverboard.serial_writer, "LED unrecognised"),
            }
        }
        b'g' => {
            if command.len() < 2 {
                return false;
            }
            match command[1] {
                b'0' => {
                    log!(hoverboard.serial_writer, "green off");
                    hoverboard.leds.green.set_low().unwrap()
                }
                b'1' => {
                    log!(hoverboard.serial_writer, "green on");
                    hoverboard.leds.green.set_high().unwrap()
                }
                _ => log!(hoverboard.serial_writer, "LED unrecognised"),
            }
        }
        b'b' => {
            let readings = hoverboard.adc_readings();
            log!(
                hoverboard.serial_writer,
                "Battery voltage: {} mV, backup: {} mV, current {} mV",
                readings.battery_voltage,
                readings.backup_battery_voltage,
                readings.motor_current
            );
        }
        b'c' => {
            if hoverboard.charge_state.is_low().unwrap() {
                log!(hoverboard.serial_writer, "Charger connected");
            } else {
                log!(hoverboard.serial_writer, "Charger not connected");
            }
        }
        b'S' => {
            if command.len() < 5 {
                return false;
            }
            let min_power = i16::from_le_bytes(command[1..3].try_into().unwrap());
            let max_power = i16::from_le_bytes(command[3..5].try_into().unwrap());
            log!(
                hoverboard.serial_writer,
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
            log!(hoverboard.serial_writer, "Spring constant {}", spring);
            *spring_constant = spring;
        }
        b'n' => {
            log!(hoverboard.serial_writer, "No target position");
            *target_position = None;
        }
        b'T' => {
            if command.len() < 9 {
                return false;
            }
            let target = i64::from_le_bytes(command[1..9].try_into().unwrap());
            log!(hoverboard.serial_writer, "Target position {}", target);
            *target_position = Some(target);
        }
        b'e' => {
            log!(hoverboard.serial_writer, "Recentre");
            hoverboard.recentre_motor();
            *target_position = Some(0);
        }
        b'+' => {
            let target = target_position.unwrap_or(0) + 10;
            log!(hoverboard.serial_writer, "Target position {}", target);
            *target_position = Some(target);
        }
        b'-' => {
            let target = target_position.unwrap_or(0) - 10;
            log!(hoverboard.serial_writer, "Target position {}", target);
            *target_position = Some(target);
        }
        b'p' => poweroff(hoverboard),
        _ => log!(
            hoverboard.serial_writer,
            "Unrecognised command {}",
            command[0]
        ),
    }
    true
}
