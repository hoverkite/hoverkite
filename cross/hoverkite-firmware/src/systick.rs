use core::sync::atomic::{AtomicU32, Ordering};
use cortex_m::peripheral::SYST;
use cortex_m_rt::exception;
use embedded_hal::blocking::delay::DelayMs;
use gd32f1x0_hal::{
    prelude::*,
    rcu::Clocks,
    timer::{CountDownTimer, Event, Timer},
};

static MILLIS_SINCE_START: AtomicU32 = AtomicU32::new(0);

#[exception]
fn SysTick() {
    MILLIS_SINCE_START.fetch_add(1, Ordering::SeqCst);
}

pub struct SysTick {
    _timer: CountDownTimer<SYST>,
}

impl SysTick {
    pub fn start(syst: SYST, clocks: &Clocks) -> Self {
        let mut timer = Timer::syst(syst, clocks).start_count_down(1.khz());
        timer.listen(Event::Update);
        Self { _timer: timer }
    }

    pub fn millis_since_start(&self) -> u32 {
        MILLIS_SINCE_START.load(Ordering::SeqCst)
    }
}

impl DelayMs<u32> for SysTick {
    fn delay_ms(&mut self, ms: u32) {
        let finish_millis = self.millis_since_start() + ms;
        while self.millis_since_start() < finish_millis {}
    }
}

impl DelayMs<u16> for SysTick {
    fn delay_ms(&mut self, ms: u16) {
        self.delay_ms(ms as u32);
    }
}

impl DelayMs<u8> for SysTick {
    fn delay_ms(&mut self, ms: u8) {
        self.delay_ms(ms as u32);
    }
}
