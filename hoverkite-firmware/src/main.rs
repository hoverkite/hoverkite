#![no_std]
#![no_main]

mod hoverboard;

use hoverboard::{Hoverboard, Leds};
// pick a panicking behavior
use panic_halt as _; // you can put a breakpoint on `rust_begin_unwind` to catch panics
                     // use panic_abort as _; // requires nightly
                     // use panic_itm as _; // logs messages over ITM; requires ITM support
                     // use panic_semihosting as _; // logs messages to the host stderr; requires a debugger

use core::{convert::Infallible, fmt::Write};
use cortex_m_rt::entry;
use embedded_hal::serial::Read;
use nb::block;
use stm32f0xx_hal::{
    gpio::{gpiob::PB2, Output, PushPull},
    pac::{self, USART2},
    prelude::*,
    serial::{Rx, Tx},
};

#[entry]
fn main() -> ! {
    // Get access to the device specific peripherals from the peripheral access crate
    let dp = pac::Peripherals::take().unwrap();

    let mut flash = dp.FLASH;
    let mut rcc = dp.RCC.configure().freeze(&mut flash);

    let mut hoverboard =
        Hoverboard::new(dp.GPIOA, dp.GPIOB, dp.GPIOC, dp.GPIOF, dp.USART2, &mut rcc);

    // Keep power on.
    hoverboard.power_latch.set_high().unwrap();

    // Split the serial struct into a receiving and a transmitting part
    let (mut tx, mut rx) = hoverboard.serial.split();

    writeln!(tx, "Ready").unwrap();
    let mut last_hall_position = None;
    loop {
        // If there is no command available to process, continue on.
        let _ = process_command(
            &mut tx,
            &mut rx,
            &mut hoverboard.leds,
            &mut hoverboard.power_latch,
        );

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
            while hoverboard.power_button.is_high().unwrap() {}
            poweroff(&mut tx, &mut hoverboard.power_latch);
        }
    }
}

fn process_command(
    tx: &mut Tx<USART2>,
    rx: &mut Rx<USART2>,
    leds: &mut Leds,
    power_latch: &mut PB2<Output<PushPull>>,
) -> nb::Result<(), Infallible> {
    let command = match nest(rx.read())? {
        Ok(v) => v,
        Err(e) => {
            writeln!(tx, "Read error {:?}", e).unwrap();
            return Ok(());
        }
    };
    match command {
        b'l' => match block!(rx.read()).unwrap() {
            b'0' => {
                writeln!(tx, "LED off").unwrap();
                leds.side.set_low().unwrap()
            }
            b'1' => {
                writeln!(tx, "LED on").unwrap();
                leds.side.set_high().unwrap()
            }
            _ => writeln!(tx, "LED unrecognised").unwrap(),
        },
        b'o' => match block!(rx.read()).unwrap() {
            b'0' => {
                writeln!(tx, "orange off").unwrap();
                leds.orange.set_low().unwrap()
            }
            b'1' => {
                writeln!(tx, "orange on").unwrap();
                leds.orange.set_high().unwrap()
            }
            _ => writeln!(tx, "LED unrecognised").unwrap(),
        },
        b'r' => match block!(rx.read()).unwrap() {
            b'0' => {
                writeln!(tx, "red off").unwrap();
                leds.red.set_low().unwrap()
            }
            b'1' => {
                writeln!(tx, "red on").unwrap();
                leds.red.set_high().unwrap()
            }
            _ => writeln!(tx, "LED unrecognised").unwrap(),
        },
        b'g' => match block!(rx.read()).unwrap() {
            b'0' => {
                writeln!(tx, "green off").unwrap();
                leds.green.set_low().unwrap()
            }
            b'1' => {
                writeln!(tx, "green on").unwrap();
                leds.green.set_high().unwrap()
            }
            _ => writeln!(tx, "LED unrecognised").unwrap(),
        },
        b'p' => poweroff(tx, power_latch),
        _ => writeln!(tx, "Unrecognised command {}", command).unwrap(),
    }
    Ok(())
}

fn nest<T, E>(result: nb::Result<T, E>) -> nb::Result<Result<T, E>, Infallible> {
    match result {
        Ok(v) => Ok(Ok(v)),
        Err(nb::Error::WouldBlock) => Err(nb::Error::WouldBlock),
        Err(nb::Error::Other(e)) => Ok(Err(e)),
    }
}

fn poweroff(tx: &mut Tx<USART2>, power_latch: &mut PB2<Output<PushPull>>) {
    writeln!(tx, "Power off").unwrap();
    power_latch.set_low().unwrap()
}
