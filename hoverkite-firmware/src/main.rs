#![no_std]
#![no_main]

mod hoverboard;

use hoverboard::{Hoverboard, Leds};
// pick a panicking behavior
use panic_halt as _; // you can put a breakpoint on `rust_begin_unwind` to catch panics
                     // use panic_abort as _; // requires nightly
                     // use panic_itm as _; // logs messages over ITM; requires ITM support
                     // use panic_semihosting as _; // logs messages to the host stderr; requires a debugger

use core::fmt::Write;
use cortex_m_rt::entry;
use embedded_hal::serial::Read;
use stm32f0xx_hal::{
    gpio::{gpiob::PB2, gpiof::PF0, Input, Output, PullUp, PushPull},
    pac::{self, USART2},
    prelude::*,
    serial::Tx,
    watchdog::Watchdog,
};

const WATCHDOG_HZ: u32 = 1;

#[entry]
fn main() -> ! {
    // Get access to the device specific peripherals from the peripheral access crate
    let dp = pac::Peripherals::take().unwrap();

    let mut flash = dp.FLASH;
    let mut rcc = dp.RCC.configure().freeze(&mut flash);

    let mut watchdog = Watchdog::new(dp.IWDG);
    watchdog.start(WATCHDOG_HZ.hz());

    let mut hoverboard =
        Hoverboard::new(dp.GPIOA, dp.GPIOB, dp.GPIOC, dp.GPIOF, dp.USART2, &mut rcc);

    // Keep power on.
    hoverboard.power_latch.set_high().unwrap();

    // If power button is pressed, wait until it is released.
    while hoverboard.power_button.is_high().unwrap() {
        watchdog.feed();
    }

    // Split the serial struct into a receiving and a transmitting part
    let (mut tx, mut rx) = hoverboard.serial.split();

    writeln!(tx, "Ready").unwrap();
    let mut last_hall_position = None;
    let mut command_buffer = [0; 5];
    let mut command_len = 0;
    loop {
        // The watchdog must be fed every second or so or the microcontroller will reset.
        watchdog.feed();

        // Read from the USART if data is available.
        match rx.read() {
            Ok(char) => {
                command_buffer[command_len] = char;
                command_len += 1;
                if process_command(
                    &command_buffer[0..command_len],
                    &mut tx,
                    &mut hoverboard.leds,
                    &mut hoverboard.power_latch,
                    &mut hoverboard.charge_state,
                ) {
                    command_len = 0;
                } else if command_len > command_buffer.len() {
                    writeln!(tx, "Command too long").unwrap();
                    command_len = 0;
                }
            }
            Err(nb::Error::WouldBlock) => {}
            Err(nb::Error::Other(e)) => writeln!(tx, "Read error {:?}", e).unwrap(),
        }

        // Log if the position from the Hall effect sensors has changed.
        let hall_position = hoverboard.hall_sensors.position();
        if hall_position != last_hall_position {
            if let Some(hall_position) = hall_position {
                writeln!(tx, "Position {}", hall_position).unwrap();
            } else {
                writeln!(tx, "Invalid position").unwrap();
            }
            last_hall_position = hall_position;
        }

        // If the power button is pressed, turn off.
        if hoverboard.power_button.is_high().unwrap() {
            // Wait until it is released.
            while hoverboard.power_button.is_high().unwrap() {
                watchdog.feed();
            }
            poweroff(&mut tx, &mut hoverboard.power_latch);
        }
    }
}

/// Process the given command, returning true if a command was successfully parsed or false if not
/// enough was read yet.
fn process_command(
    command: &[u8],
    tx: &mut Tx<USART2>,
    leds: &mut Leds,
    power_latch: &mut PB2<Output<PushPull>>,
    charge_state: &mut PF0<Input<PullUp>>,
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
                    writeln!(tx, "LED off").unwrap();
                    leds.side.set_low().unwrap()
                }
                b'1' => {
                    writeln!(tx, "LED on").unwrap();
                    leds.side.set_high().unwrap()
                }
                _ => writeln!(tx, "LED unrecognised").unwrap(),
            }
        }
        b'o' => {
            if command.len() < 2 {
                return false;
            }
            match command[1] {
                b'0' => {
                    writeln!(tx, "orange off").unwrap();
                    leds.orange.set_low().unwrap()
                }
                b'1' => {
                    writeln!(tx, "orange on").unwrap();
                    leds.orange.set_high().unwrap()
                }
                _ => writeln!(tx, "LED unrecognised").unwrap(),
            }
        }
        b'r' => {
            if command.len() < 2 {
                return false;
            }
            match command[1] {
                b'0' => {
                    writeln!(tx, "red off").unwrap();
                    leds.red.set_low().unwrap()
                }
                b'1' => {
                    writeln!(tx, "red on").unwrap();
                    leds.red.set_high().unwrap()
                }
                _ => writeln!(tx, "LED unrecognised").unwrap(),
            }
        }
        b'g' => {
            if command.len() < 2 {
                return false;
            }
            match command[1] {
                b'0' => {
                    writeln!(tx, "green off").unwrap();
                    leds.green.set_low().unwrap()
                }
                b'1' => {
                    writeln!(tx, "green on").unwrap();
                    leds.green.set_high().unwrap()
                }
                _ => writeln!(tx, "LED unrecognised").unwrap(),
            }
        }
        b'c' => {
            if charge_state.is_low().unwrap() {
                writeln!(tx, "Charger connected").unwrap();
            } else {
                writeln!(tx, "Charger not connected").unwrap();
            }
        }
        b'p' => poweroff(tx, power_latch),
        _ => writeln!(tx, "Unrecognised command {}", command[0]).unwrap(),
    }
    true
}

fn poweroff(tx: &mut Tx<USART2>, power_latch: &mut PB2<Output<PushPull>>) {
    writeln!(tx, "Power off").unwrap();
    power_latch.set_low().unwrap()
}
