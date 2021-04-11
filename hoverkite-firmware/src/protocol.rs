use crate::hoverboard::Hoverboard;
use crate::poweroff;
use core::{convert::TryInto, fmt::Write, ops::RangeInclusive};
use gd32f1x0_hal::prelude::*;

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
                    writeln!(hoverboard.serial_writer, "LED off").unwrap();
                    hoverboard.leds.side.set_low().unwrap()
                }
                b'1' => {
                    writeln!(hoverboard.serial_writer, "LED on").unwrap();
                    hoverboard.leds.side.set_high().unwrap()
                }
                _ => writeln!(hoverboard.serial_writer, "LED unrecognised").unwrap(),
            }
        }
        b'o' => {
            if command.len() < 2 {
                return false;
            }
            match command[1] {
                b'0' => {
                    writeln!(hoverboard.serial_writer, "orange off").unwrap();
                    hoverboard.leds.orange.set_low().unwrap()
                }
                b'1' => {
                    writeln!(hoverboard.serial_writer, "orange on").unwrap();
                    hoverboard.leds.orange.set_high().unwrap()
                }
                _ => writeln!(hoverboard.serial_writer, "LED unrecognised").unwrap(),
            }
        }
        b'r' => {
            if command.len() < 2 {
                return false;
            }
            match command[1] {
                b'0' => {
                    writeln!(hoverboard.serial_writer, "red off").unwrap();
                    hoverboard.leds.red.set_low().unwrap()
                }
                b'1' => {
                    writeln!(hoverboard.serial_writer, "red on").unwrap();
                    hoverboard.leds.red.set_high().unwrap()
                }
                _ => writeln!(hoverboard.serial_writer, "LED unrecognised").unwrap(),
            }
        }
        b'g' => {
            if command.len() < 2 {
                return false;
            }
            match command[1] {
                b'0' => {
                    writeln!(hoverboard.serial_writer, "green off").unwrap();
                    hoverboard.leds.green.set_low().unwrap()
                }
                b'1' => {
                    writeln!(hoverboard.serial_writer, "green on").unwrap();
                    hoverboard.leds.green.set_high().unwrap()
                }
                _ => writeln!(hoverboard.serial_writer, "LED unrecognised").unwrap(),
            }
        }
        b'b' => {
            let readings = hoverboard.adc_readings();
            writeln!(
                hoverboard.serial_writer,
                "Battery voltage: {} mV, backup: {} mV, current {} mV",
                readings.battery_voltage, readings.backup_battery_voltage, readings.motor_current
            )
            .unwrap();
        }
        b'c' => {
            if hoverboard.charge_state.is_low().unwrap() {
                writeln!(hoverboard.serial_writer, "Charger connected").unwrap();
            } else {
                writeln!(hoverboard.serial_writer, "Charger not connected").unwrap();
            }
        }
        b'S' => {
            if command.len() < 5 {
                return false;
            }
            let min_power = i16::from_le_bytes(command[1..3].try_into().unwrap());
            let max_power = i16::from_le_bytes(command[3..5].try_into().unwrap());
            writeln!(
                hoverboard.serial_writer,
                "max speed {}..{}",
                min_power, max_power
            )
            .unwrap();
            *speed_limits = min_power..=max_power;
        }
        b'K' => {
            if command.len() < 3 {
                return false;
            }
            let spring = u16::from_le_bytes(command[1..3].try_into().unwrap()).into();
            writeln!(hoverboard.serial_writer, "Spring constant {}", spring).unwrap();
            *spring_constant = spring;
        }
        b'n' => {
            writeln!(hoverboard.serial_writer, "No target position").unwrap();
            *target_position = None;
        }
        b'T' => {
            if command.len() < 9 {
                return false;
            }
            let target = i64::from_le_bytes(command[1..9].try_into().unwrap());
            writeln!(hoverboard.serial_writer, "Target position {}", target).unwrap();
            *target_position = Some(target);
        }
        b'e' => {
            writeln!(hoverboard.serial_writer, "Recentre").unwrap();
            hoverboard.recentre_motor();
            *target_position = Some(0);
        }
        b'+' => {
            let target = target_position.unwrap_or(0) + 10;
            writeln!(hoverboard.serial_writer, "Target position {}", target).unwrap();
            *target_position = Some(target);
        }
        b'-' => {
            let target = target_position.unwrap_or(0) - 10;
            writeln!(hoverboard.serial_writer, "Target position {}", target).unwrap();
            *target_position = Some(target);
        }
        b'p' => poweroff(hoverboard),
        _ => writeln!(
            hoverboard.serial_writer,
            "Unrecognised command {}",
            command[0]
        )
        .unwrap(),
    }
    true
}
