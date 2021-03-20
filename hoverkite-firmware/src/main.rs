#![no_std]
#![no_main]

mod hoverboard;

use hoverboard::Hoverboard;
// pick a panicking behavior
use panic_halt as _; // you can put a breakpoint on `rust_begin_unwind` to catch panics
                     // use panic_abort as _; // requires nightly
                     // use panic_itm as _; // logs messages over ITM; requires ITM support
                     // use panic_semihosting as _; // logs messages to the host stderr; requires a debugger

use core::fmt::{Debug, Write};
use core::str::from_utf8;
use cortex_m_rt::entry;
use embedded_hal::serial::Read;
use nb::block;
use stm32f0xx_hal::{pac, prelude::*};

#[entry]
fn main() -> ! {
    // Get access to the device specific peripherals from the peripheral access crate
    let dp = pac::Peripherals::take().unwrap();

    let mut flash = dp.FLASH;
    let mut rcc = dp.RCC.configure().freeze(&mut flash);

    let mut hoverboard = Hoverboard::new(dp.GPIOA, dp.GPIOB, dp.USART2, &mut rcc);

    // Split the serial struct into a receiving and a transmitting part
    let (mut tx, mut rx) = hoverboard.serial.split();

    let mut line = [0; 20];
    writeln!(tx, "Ready").unwrap();
    loop {
        let length = read_line(&mut rx, &mut line);
        let line_str = from_utf8(&line[0..length]).unwrap();
        writeln!(tx, "got '{}'", line_str).unwrap();
        match line[0] {
            b'l' => match line[1] {
                b'0' => {
                    writeln!(tx, "LED off").unwrap();
                    hoverboard.side_led.set_low().unwrap()
                }
                b'1' => {
                    writeln!(tx, "LED on").unwrap();
                    hoverboard.side_led.set_high().unwrap()
                }
                _ => writeln!(tx, "LED unrecognised").unwrap(),
            },
            b'o' => match line[1] {
                b'0' => {
                    writeln!(tx, "orange off").unwrap();
                    hoverboard.orange_led.set_low().unwrap()
                }
                b'1' => {
                    writeln!(tx, "orange on").unwrap();
                    hoverboard.orange_led.set_high().unwrap()
                }
                _ => writeln!(tx, "LED unrecognised").unwrap(),
            },
            b'r' => match line[1] {
                b'0' => {
                    writeln!(tx, "red off").unwrap();
                    hoverboard.red_led.set_low().unwrap()
                }
                b'1' => {
                    writeln!(tx, "red on").unwrap();
                    hoverboard.red_led.set_high().unwrap()
                }
                _ => writeln!(tx, "LED unrecognised").unwrap(),
            },
            b'g' => match line[1] {
                b'0' => {
                    writeln!(tx, "green off").unwrap();
                    hoverboard.green_led.set_low().unwrap()
                }
                b'1' => {
                    writeln!(tx, "green on").unwrap();
                    hoverboard.green_led.set_high().unwrap()
                }
                _ => writeln!(tx, "LED unrecognised").unwrap(),
            },
            _ => writeln!(tx, "Unrecognised command").unwrap(),
        }
    }
}

fn read_line<R>(rx: &mut R, buf: &mut [u8]) -> usize
where
    R: Read<u8>,
    R::Error: Debug,
{
    for i in 0..buf.len() {
        buf[i] = block!(rx.read()).unwrap();
        if buf[i] == b'\n' || buf[i] == b'\r' {
            return i;
        }
    }
    buf.len()
}
