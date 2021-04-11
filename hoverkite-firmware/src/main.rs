#![no_std]
#![no_main]

mod buffered_tx;
mod hoverboard;
mod motor;
mod protocol;
mod util;

// pick a panicking behavior
use panic_halt as _; // you can put a breakpoint on `rust_begin_unwind` to catch panics
                     // use panic_abort as _; // requires nightly
                     // use panic_itm as _; // logs messages over ITM; requires ITM support
                     // use panic_semihosting as _; // logs messages to the host stderr; requires a debugger

use cortex_m_rt::entry;
use embedded_hal::serial::Read;
use gd32f1x0_hal::{pac, prelude::*, watchdog::FreeWatchdog};
use hoverboard::Hoverboard;
use protocol::{process_command, send_position, HoverboardExt};
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

    log!(
        hoverboard.response_tx(),
        "System clock {} Hz",
        clocks.sysclk().0
    );
    log!(
        hoverboard.response_tx(),
        "ADC clock {} Hz",
        clocks.adcclk().0
    );

    // If power button is pressed, wait until it is released.
    while hoverboard.power_button.is_high().unwrap() {
        watchdog.feed();
    }

    log!(hoverboard.response_tx(), "Ready");

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
        match hoverboard.command_rx().read() {
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
                } else if command_len >= command_buffer.len() {
                    log!(hoverboard.response_tx(), "Command too long");
                    command_len = 0;
                }
            }
            Err(nb::Error::WouldBlock) => {}
            Err(nb::Error::Other(e)) => {
                log!(
                    hoverboard.response_tx(),
                    "Read error {:?}, dropping {} bytes",
                    e,
                    command_len
                );
                command_len = 0;
            }
        }

        // Log if the position has changed.
        let position = hoverboard.motor_position();
        if position != last_position {
            send_position(hoverboard.response_tx(), position);
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

pub fn poweroff(hoverboard: &mut Hoverboard) {
    log!(hoverboard.response_tx(), "Power off");
    hoverboard.power_latch.set_low().unwrap()
}
