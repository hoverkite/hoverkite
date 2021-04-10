#![no_std]
#![no_main]

mod buffered_tx;
mod hoverboard;
mod motor;
mod util;

// pick a panicking behavior
use panic_halt as _; // you can put a breakpoint on `rust_begin_unwind` to catch panics
                     // use panic_abort as _; // requires nightly
                     // use panic_itm as _; // logs messages over ITM; requires ITM support
                     // use panic_semihosting as _; // logs messages to the host stderr; requires a debugger

use core::{convert::TryInto, fmt::Write, ops::RangeInclusive};
use cortex_m_rt::entry;
use embedded_hal::serial::Read;
use gd32f1x0_hal::{pac, prelude::*, watchdog::FreeWatchdog};
use hoverboard::Hoverboard;
use util::clamp;

const WATCHDOG_MILLIS: u32 = 1000;

#[entry]
fn main() -> ! {
    // Get access to the device specific peripherals from the peripheral access crate
    let dp = pac::Peripherals::take().unwrap();

    let mut rcu = dp.RCU.constrain();
    let mut flash = dp.FMC.constrain();
    let clocks = rcu
        .cfgr
        .sysclk(72.mhz())
        .adcclk(12.mhz())
        .freeze(&mut flash.ws);

    let mut watchdog = FreeWatchdog::new(dp.FWDGT);
    watchdog.start(WATCHDOG_MILLIS.ms());

    let mut hoverboard = Hoverboard::new(
        dp.GPIOA,
        dp.GPIOB,
        dp.GPIOC,
        dp.GPIOF,
        dp.USART0,
        dp.USART1,
        dp.TIMER0,
        dp.DMA,
        dp.ADC,
        &mut rcu.ahb,
        &mut rcu.apb1,
        &mut rcu.apb2,
        clocks,
    );

    // Keep power on.
    hoverboard.power_latch.set_high().unwrap();

    writeln!(
        hoverboard.serial_writer,
        "System clock {} Hz",
        clocks.sysclk().0
    )
    .unwrap();
    writeln!(
        hoverboard.serial_writer,
        "ADC clock {} Hz",
        clocks.adcclk().0
    )
    .unwrap();

    // If power button is pressed, wait until it is released.
    while hoverboard.power_button.is_high().unwrap() {
        watchdog.feed();
    }

    writeln!(hoverboard.serial_writer, "Ready").unwrap();

    let mut last_position = 0;
    let mut command_buffer = [0; 10];
    let mut command_len = 0;
    let mut speed;
    let mut target_position: Option<i64> = None;
    let mut speed_limits = -200..=200;
    let mut spring_constant = 10;
    loop {
        // The watchdog must be fed every second or so or the microcontroller will reset.
        watchdog.feed();

        // Read from the USART if data is available.
        match hoverboard.serial_rx.read() {
            Ok(char) => {
                command_buffer[command_len] = char;
                command_len += 1;
                if process_command(
                    &command_buffer[0..command_len],
                    &mut hoverboard,
                    &mut speed_limits,
                    &mut target_position,
                    &mut spring_constant,
                ) {
                    command_len = 0;
                } else if command_len > command_buffer.len() {
                    writeln!(hoverboard.serial_writer, "Command too long").unwrap();
                    command_len = 0;
                }
            }
            Err(nb::Error::WouldBlock) => {}
            Err(nb::Error::Other(e)) => {
                writeln!(
                    hoverboard.serial_writer,
                    "Read error {:?}, dropping {} bytes",
                    e, command_len
                )
                .unwrap();
                command_len = 0;
            }
        }

        // Log if the position has changed.
        let position = hoverboard.motor_position();
        if position != last_position {
            writeln!(hoverboard.serial_writer, "Position {}", position).unwrap();
            last_position = position;
        }

        // Try to move towards the target position.
        if let Some(target_position) = target_position {
            let difference = target_position - position;
            speed = clamp(difference * spring_constant, &speed_limits);

            // Set LEDs based on position difference
            if difference.abs() < 3 {
                hoverboard.leds.green.set_high().unwrap();
                hoverboard.leds.orange.set_low().unwrap();
                hoverboard.leds.red.set_low().unwrap();
            } else if difference > 0 {
                hoverboard.leds.green.set_low().unwrap();
                hoverboard.leds.orange.set_high().unwrap();
                hoverboard.leds.red.set_low().unwrap();
            } else {
                hoverboard.leds.green.set_low().unwrap();
                hoverboard.leds.orange.set_low().unwrap();
                hoverboard.leds.red.set_high().unwrap();
            }
        } else {
            speed = 0;

            hoverboard.leds.green.set_low().unwrap();
            hoverboard.leds.orange.set_low().unwrap();
            hoverboard.leds.red.set_low().unwrap();
        }

        // Drive the motor.
        hoverboard.set_motor_power(speed);

        // If the power button is pressed, turn off.
        if hoverboard.power_button.is_high().unwrap() {
            // Wait until it is released.
            while hoverboard.power_button.is_high().unwrap() {
                watchdog.feed();
            }
            poweroff(&mut hoverboard);
        }
    }
}

/// Process the given command, returning true if a command was successfully parsed or false if not
/// enough was read yet.
fn process_command(
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
        b's' => {
            if command.len() < 2 {
                return false;
            }
            let power = char_to_digit::<i16>(command[1]) * 30;
            writeln!(hoverboard.serial_writer, "max speed {}", power).unwrap();
            *speed_limits = -power..=power;
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
        b'k' => {
            if command.len() < 2 {
                return false;
            }
            let spring = char_to_digit::<i64>(command[1]) * 2;
            writeln!(hoverboard.serial_writer, "Spring constant {}", spring).unwrap();
            *spring_constant = spring;
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
        b't' => {
            if command.len() < 2 {
                return false;
            }
            let target = char_to_digit::<i64>(command[1]) * 100;
            writeln!(hoverboard.serial_writer, "Target position {}", target).unwrap();
            *target_position = Some(target);
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

fn char_to_digit<T: From<u8>>(char: u8) -> T {
    match char {
        b'0' => 0,
        b'1' => 1,
        b'2' => 2,
        b'3' => 3,
        b'4' => 4,
        b'5' => 5,
        b'6' => 6,
        b'7' => 7,
        b'8' => 8,
        b'9' => 9,
        _ => 0,
    }
    .into()
}

fn poweroff(hoverboard: &mut Hoverboard) {
    writeln!(hoverboard.serial_writer, "Power off").unwrap();
    hoverboard.power_latch.set_low().unwrap()
}
