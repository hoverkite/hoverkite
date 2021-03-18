#![no_std]
#![no_main]

// pick a panicking behavior
use panic_halt as _; // you can put a breakpoint on `rust_begin_unwind` to catch panics
                     // use panic_abort as _; // requires nightly
                     // use panic_itm as _; // logs messages over ITM; requires ITM support
                     // use panic_semihosting as _; // logs messages to the host stderr; requires a debugger

use core::fmt::Write;
use cortex_m_rt::entry;
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

    // Prepare the alternate function I/O registers
    let (tx, rx) = cortex_m::interrupt::free(move |cs| {
        (
            gpioa.pa2.into_alternate_af1(cs),
            gpioa.pa3.into_alternate_af1(cs),
        )
    });

    // Set up the usart device. Takes ownership over the USART register and tx/rx pins. The rest of
    // the registers are used to enable and configure the device.
    let serial = Serial::usart2(dp.USART2, (tx, rx), 115200.bps(), &mut rcc);

    // Split the serial struct into a receiving and a transmitting part
    let (mut tx, _rx) = serial.split();

    loop {
        // your code goes here
        writeln!(tx, "Hello!").unwrap();
    }
}
