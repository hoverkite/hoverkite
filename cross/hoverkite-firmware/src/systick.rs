use core::sync::atomic::{AtomicU32, Ordering};
use cortex_m::peripheral::SYST;
use cortex_m_rt::exception;
use embedded_hal::delay::DelayNs;
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

impl DelayNs for SysTick {
    fn delay_ns(&mut self, ns: u32) {
        let finish_millis = self.millis_since_start() + ns / 1_000_000;
        while self.millis_since_start() < finish_millis {}
    }
}
