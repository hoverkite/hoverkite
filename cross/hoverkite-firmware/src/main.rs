#![no_std]
#![no_main]

mod buffered_tx;
mod circular_buffer;
mod hoverboard;
mod motor;
mod protocol;
mod pwm;
mod systick;
mod util;

#[cfg(feature = "primary")]
use messages::Command;
use messages::SideResponse;
#[cfg(feature = "secondary")]
use messages::{Note, Response};
// pick a panicking behavior
use panic_halt as _; // you can put a breakpoint on `rust_begin_unwind` to catch panics
                     // use panic_abort as _; // requires nightly
                     // use panic_itm as _; // logs messages over ITM; requires ITM support
                     // use panic_semihosting as _; // logs messages to the host stderr; requires a debugger

use circular_buffer::CircularBuffer;
#[cfg(feature = "secondary")]
use core::num::NonZeroU32;
use cortex_m_rt::entry;
use embedded_hal::serial::Read;
use gd32f1x0_hal::{pac, prelude::*, time::Hertz, watchdog::FreeWatchdog};
use hoverboard::Hoverboard;
#[cfg(feature = "primary")]
use protocol::process_response;
use protocol::{process_command, send_position, HoverboardExt};
use systick::SysTick;
use util::clamp;

#[cfg(feature = "secondary")]
use crate::protocol::THIS_SIDE;

const WATCHDOG_MILLIS: u32 = 1000;

#[cfg(feature = "secondary")]
const POWER_ON_TUNE: [Note; 2] = [
    Note {
        frequency: NonZeroU32::new(1000),
        duration_ms: 200,
    },
    Note {
        frequency: NonZeroU32::new(2000),
        duration_ms: 100,
    },
];

/// Frequency of tone to play while powering off. We can't easily play a tune because the main loop
/// is no longer running by then.
#[cfg(feature = "secondary")]
const POWER_OFF_FREQUENCY: Hertz = Hertz(800);

/// If the power button is held for more than this duration then don't play the power on tune.
#[cfg(feature = "secondary")]
const POWER_ON_SILENT_MS: u32 = 1000;

#[entry]
fn main() -> ! {
    let cp = cortex_m::Peripherals::take().unwrap();
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

    let systick = SysTick::start(cp.SYST, &clocks);

    let mut hoverboard = Hoverboard::new(
        dp.GPIOA,
        dp.GPIOB,
        dp.GPIOC,
        dp.GPIOF,
        dp.USART0,
        dp.USART1,
        dp.TIMER0,
        dp.TIMER1,
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

    let mut note_queue = CircularBuffer::<_, 100>::default();
    // The timestamp at which to start playing the next note.
    let mut next_note_time = 0;

    #[cfg(feature = "secondary")]
    if systick.millis_since_start() < POWER_ON_SILENT_MS {
        note_queue.add_all(&POWER_ON_TUNE);
    }

    log!(hoverboard.response_tx(), "Ready");

    let mut last_position = 0;
    let mut command_buffer = [0; 12];
    let mut command_len = 0;
    #[cfg(feature = "primary")]
    let mut proxy_response_buffer = [0; 100];
    #[cfg(feature = "primary")]
    let mut proxy_response_length = 0;
    let mut speed;
    let mut target_position: Option<i64> = None;
    let mut speed_limits = -200..=200;
    let mut spring_constant = 10;
    loop {
        // The watchdog must be fed every second or so or the microcontroller will reset.
        watchdog.feed();

        // Read from the command USART if data is available.
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
                    &mut note_queue,
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

        // Read from the secondary USART if data is available
        #[cfg(feature = "primary")]
        match hoverboard.serial_rx.read() {
            Ok(char) => {
                proxy_response_buffer[proxy_response_length] = char;
                proxy_response_length += 1;
                if process_response(
                    &proxy_response_buffer[0..proxy_response_length],
                    &mut hoverboard,
                ) {
                    proxy_response_length = 0;
                } else if proxy_response_length >= proxy_response_buffer.len() {
                    log!(hoverboard.response_tx(), "Secondary response too long");
                    proxy_response_length = 0;
                }
            }
            Err(nb::Error::WouldBlock) => {}
            Err(nb::Error::Other(e)) => {
                log!(
                    hoverboard.response_tx(),
                    "Read error on secondary {:?}, dropping {} bytes",
                    e,
                    proxy_response_length
                );
                proxy_response_length = 0;
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
            if difference.abs() < 5 {
                hoverboard.leds.side.set_low().unwrap();
            } else {
                hoverboard.leds.side.set_high().unwrap();
            }
        } else {
            speed = 0;

            hoverboard.leds.green.set_low().unwrap();
            hoverboard.leds.orange.set_low().unwrap();
            hoverboard.leds.red.set_low().unwrap();
            hoverboard.leds.side.set_low().unwrap();
        }

        // Drive the motor.
        hoverboard.set_motor_power(speed);

        let current_time = systick.millis_since_start();
        if current_time > next_note_time {
            // Play the next note on the buzzer, or turn it off if there is none.
            let note = note_queue.take().unwrap_or_default();
            if note.frequency.is_some() {
                log!(hoverboard.response_tx(), "Playing {}", note);
            }
            hoverboard
                .buzzer
                .set_frequency(note.frequency.map(|frequency| frequency.get().hz()));
            next_note_time = current_time + note.duration_ms;
        }

        // If the power button is pressed, turn off.
        if hoverboard.power_button.is_high().unwrap() {
            #[cfg(feature = "secondary")]
            hoverboard.buzzer.set_frequency(Some(POWER_OFF_FREQUENCY));
            // Wait until it is released.
            while hoverboard.power_button.is_high().unwrap() {
                watchdog.feed();
            }
            #[cfg(feature = "secondary")]
            {
                log!(hoverboard.response_tx(), "Telling primary to power off");
                // Tell primary to power off, but only here in response to the power button press.
                SideResponse {
                    side: THIS_SIDE,
                    response: Response::PowerOff,
                }
                .write_to(&mut hoverboard.serial_writer)
                .unwrap()
            }
            poweroff(&mut hoverboard);
        }
    }
}

pub fn poweroff(hoverboard: &mut Hoverboard) {
    #[cfg(feature = "primary")]
    {
        log!(hoverboard.response_tx(), "Telling secondary to power off");
        // Ensure secondary powers off before we do.
        Command::PowerOff
            .write_to(&mut hoverboard.serial_writer)
            .unwrap();
        hoverboard.serial_writer.bflush().unwrap();
    }
    log!(hoverboard.response_tx(), "Power off");
    hoverboard.power_latch.set_low().unwrap();
    log!(hoverboard.response_tx(), "Powered off");
}
