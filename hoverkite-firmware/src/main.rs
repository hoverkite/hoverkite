#![no_std]
#![no_main]

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
use stm32f0xx_hal::{pac, prelude::*, serial::Serial};

#[entry]
fn main() -> ! {
    // Get access to the core peripherals from the cortex-m crate
    let cp = cortex_m::Peripherals::take().unwrap();
    // Get access to the device specific peripherals from the peripheral access crate
    let dp = pac::Peripherals::take().unwrap();

    let mut flash = dp.FLASH;
    let mut rcc = dp.RCC.configure().freeze(&mut flash);

    let gpioa = dp.GPIOA.split(&mut rcc);
    let gpiob = dp.GPIOB.split(&mut rcc);

    // NB: Don't try to use pa13, that's SWDIO
    let pa0 = gpioa.pa0;
    let pa15 = gpioa.pa15;
    let pa12 = gpioa.pa12;
    let pb3 = gpiob.pb3;
    let (mut side_led, mut green_led, mut orange_led, mut red_led) =
        cortex_m::interrupt::free(|cs| {
            (
                pa0.into_push_pull_output(cs),
                pa15.into_push_pull_output(cs),
                pa12.into_push_pull_output(cs),
                pb3.into_push_pull_output(cs),
            )
        });

    // Prepare the alternate function I/O registers for the USART.
    let pa2 = gpioa.pa2;
    let pa3 = gpioa.pa3;
    let (tx, rx) = cortex_m::interrupt::free(move |cs| {
        (pa2.into_alternate_af1(cs), pa3.into_alternate_af1(cs))
    });
    // Set up the usart device. Takes ownership over the USART register and tx/rx pins. The rest of
    // the registers are used to enable and configure the device.
    let serial = Serial::usart2(dp.USART2, (tx, rx), 115200.bps(), &mut rcc);

    // Split the serial struct into a receiving and a transmitting part
    let (mut tx, mut rx) = serial.split();

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
                    side_led.set_low().unwrap()
                }
                b'1' => {
                    writeln!(tx, "LED on").unwrap();
                    side_led.set_high().unwrap()
                }
                _ => writeln!(tx, "LED unrecognised").unwrap(),
            },
            b'o' => match line[1] {
                b'0' => {
                    writeln!(tx, "orange off").unwrap();
                    orange_led.set_low().unwrap()
                }
                b'1' => {
                    writeln!(tx, "orange on").unwrap();
                    orange_led.set_high().unwrap()
                }
                _ => writeln!(tx, "LED unrecognised").unwrap(),
            },
            b'r' => match line[1] {
                b'0' => {
                    writeln!(tx, "red off").unwrap();
                    red_led.set_low().unwrap()
                }
                b'1' => {
                    writeln!(tx, "red on").unwrap();
                    red_led.set_high().unwrap()
                }
                _ => writeln!(tx, "LED unrecognised").unwrap(),
            },
            b'g' => match line[1] {
                b'0' => {
                    writeln!(tx, "green off").unwrap();
                    green_led.set_low().unwrap()
                }
                b'1' => {
                    writeln!(tx, "green on").unwrap();
                    green_led.set_high().unwrap()
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
