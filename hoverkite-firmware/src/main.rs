#![no_std]
#![no_main]

mod hoverboard;

use hoverboard::Hoverboard;
// pick a panicking behavior
use panic_halt as _; // you can put a breakpoint on `rust_begin_unwind` to catch panics
                     // use panic_abort as _; // requires nightly
                     // use panic_itm as _; // logs messages over ITM; requires ITM support
                     // use panic_semihosting as _; // logs messages to the host stderr; requires a debugger

use core::fmt::Write;
use cortex_m_rt::entry;
use embedded_hal::serial::Read;
use gd32f1x0_hal::{pac, prelude::*, watchdog::FreeWatchdog};

const WATCHDOG_MILLIS: u32 = 1000;

#[entry]
fn main() -> ! {
    // Get access to the device specific peripherals from the peripheral access crate
    let dp = pac::Peripherals::take().unwrap();

    let mut rcu = dp.RCU.constrain();
    let clocks = rcu
        .cfgr
        .sysclk(72.mhz())
        .adcclk(12.mhz())
        .freeze(&dp.FMC.ws);

    let mut watchdog = FreeWatchdog::new(dp.FWDGT);
    watchdog.start(WATCHDOG_MILLIS.ms());

    let mut hoverboard = Hoverboard::new(
        dp.GPIOA,
        dp.GPIOB,
        dp.GPIOC,
        dp.GPIOF,
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

    writeln!(hoverboard.serial, "System clock {} Hz", clocks.sysclk().0).unwrap();
    writeln!(hoverboard.serial, "ADC clock {} Hz", clocks.adcclk().0).unwrap();

    // If power button is pressed, wait until it is released.
    while hoverboard.power_button.is_high().unwrap() {
        watchdog.feed();
    }

    writeln!(hoverboard.serial, "Ready").unwrap();

    let mut last_hall_position = None;
    let mut command_buffer = [0; 5];
    let mut command_len = 0;
    let mut speed = 0;
    let mut position: i64 = 0;
    let mut target_position: Option<i64> = None;
    let target_speed = 60;
    loop {
        // The watchdog must be fed every second or so or the microcontroller will reset.
        watchdog.feed();

        // Read from the USART if data is available.
        match hoverboard.serial.read() {
            Ok(char) => {
                command_buffer[command_len] = char;
                command_len += 1;
                if process_command(
                    &command_buffer[0..command_len],
                    &mut hoverboard,
                    &mut speed,
                    &mut target_position,
                ) {
                    command_len = 0;
                } else if command_len > command_buffer.len() {
                    writeln!(hoverboard.serial, "Command too long").unwrap();
                    command_len = 0;
                }
            }
            Err(nb::Error::WouldBlock) => {}
            Err(nb::Error::Other(e)) => writeln!(hoverboard.serial, "Read error {:?}", e).unwrap(),
        }

        // Log if the position from the Hall effect sensors has changed.
        let hall_position = hoverboard.hall_sensors.position();
        if hall_position != last_hall_position {
            if let Some(hall_position) = hall_position {
                if let Some(last_hall_position) = last_hall_position {
                    // Update absolute position
                    let difference = (6 + hall_position - last_hall_position) % 6;
                    match difference {
                        0 => {}
                        1 => position += 1,
                        2 => position += 2,
                        4 => position -= 2,
                        5 => position -= 1,
                        _ => {
                            writeln!(hoverboard.serial, "Wheel moved too much");
                        }
                    }
                }
                writeln!(
                    hoverboard.serial,
                    "Position {} ({})",
                    hall_position, position
                )
                .unwrap();
                last_hall_position = Some(hall_position);
            } else {
                writeln!(hoverboard.serial, "Invalid position").unwrap();
            }
        }

        // Try to move towards the target position.
        if let Some(target_position) = target_position {
            speed = if target_position < position {
                -target_speed
            } else if target_position > position {
                target_speed
            } else {
                0
            };
        }

        // Drive the motor.
        if let Some(hall_position) = hall_position {
            hoverboard.motor.set_position_power(speed, hall_position);
        }

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
    speed: &mut i16,
    target_position: &mut Option<i64>,
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
                    writeln!(hoverboard.serial, "LED off").unwrap();
                    hoverboard.leds.side.set_low().unwrap()
                }
                b'1' => {
                    writeln!(hoverboard.serial, "LED on").unwrap();
                    hoverboard.leds.side.set_high().unwrap()
                }
                _ => writeln!(hoverboard.serial, "LED unrecognised").unwrap(),
            }
        }
        b'o' => {
            if command.len() < 2 {
                return false;
            }
            match command[1] {
                b'0' => {
                    writeln!(hoverboard.serial, "orange off").unwrap();
                    hoverboard.leds.orange.set_low().unwrap()
                }
                b'1' => {
                    writeln!(hoverboard.serial, "orange on").unwrap();
                    hoverboard.leds.orange.set_high().unwrap()
                }
                _ => writeln!(hoverboard.serial, "LED unrecognised").unwrap(),
            }
        }
        b'r' => {
            if command.len() < 2 {
                return false;
            }
            match command[1] {
                b'0' => {
                    writeln!(hoverboard.serial, "red off").unwrap();
                    hoverboard.leds.red.set_low().unwrap()
                }
                b'1' => {
                    writeln!(hoverboard.serial, "red on").unwrap();
                    hoverboard.leds.red.set_high().unwrap()
                }
                _ => writeln!(hoverboard.serial, "LED unrecognised").unwrap(),
            }
        }
        b'g' => {
            if command.len() < 2 {
                return false;
            }
            match command[1] {
                b'0' => {
                    writeln!(hoverboard.serial, "green off").unwrap();
                    hoverboard.leds.green.set_low().unwrap()
                }
                b'1' => {
                    writeln!(hoverboard.serial, "green on").unwrap();
                    hoverboard.leds.green.set_high().unwrap()
                }
                _ => writeln!(hoverboard.serial, "LED unrecognised").unwrap(),
            }
        }
        b'b' => {
            let readings = hoverboard.adc_readings();
            writeln!(
                hoverboard.serial,
                "Battery voltage: {} mV, backup: {} mV, current {} mV",
                readings.battery_voltage, readings.backup_battery_voltage, readings.motor_current
            )
            .unwrap();
        }
        b'c' => {
            if hoverboard.charge_state.is_low().unwrap() {
                writeln!(hoverboard.serial, "Charger connected").unwrap();
            } else {
                writeln!(hoverboard.serial, "Charger not connected").unwrap();
            }
        }
        b'm' => {
            if command.len() < 3 {
                return false;
            }
            let position = match command[1] {
                b'0' => 0,
                b'1' => 1,
                b'2' => 2,
                b'3' => 3,
                b'4' => 4,
                b'5' => 5,
                _ => 6,
            };
            let power = match command[2] {
                b'1' => 10,
                b'9' => 90,
                _ => 0,
            };
            writeln!(
                hoverboard.serial,
                "motor position {} power {}",
                position, power
            )
            .unwrap();
            hoverboard.motor.set_position_power(power, position);
        }
        b's' | b'v' => {
            if command.len() < 2 {
                return false;
            }
            let mut power = match command[1] {
                b'1' => 10,
                b'2' => 20,
                b'3' => 30,
                b'4' => 40,
                b'5' => 50,
                b'6' => 60,
                b'7' => 70,
                b'8' => 80,
                b'9' => 90,
                _ => 0,
            };
            if command[0] == b'v' {
                power = -power;
            }
            writeln!(hoverboard.serial, "motor speed {}", power).unwrap();
            *speed = power;
            *target_position = None;
        }
        b't' => {
            if command.len() < 2 {
                return false;
            }
            let target = match command[1] {
                b'0' => 0,
                b'1' => 10,
                b'2' => 20,
                b'3' => 30,
                b'4' => 40,
                b'5' => 50,
                b'6' => 60,
                b'7' => 70,
                b'8' => 80,
                b'9' => 90,
                _ => 0,
            };
            writeln!(hoverboard.serial, "Target position {}", target).unwrap();
            *target_position = Some(target);
        }
        b'p' => poweroff(hoverboard),
        _ => writeln!(hoverboard.serial, "Unrecognised command {}", command[0]).unwrap(),
    }
    true
}

fn poweroff(hoverboard: &mut Hoverboard) {
    writeln!(hoverboard.serial, "Power off").unwrap();
    hoverboard.power_latch.set_low().unwrap()
}
