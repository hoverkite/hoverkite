use embedded_hal_02::Pwm as _;
use gd32f1x0_hal::{
    gpio::{
        gpioa::{PA0, PA1, PA3},
        gpiob::PB10,
        Alternate, AF2,
    },
    pac::TIMER1,
    prelude::*,
    pwm::{Channel, Pwm},
    rcu::{Clocks, APB1},
    time::Hertz,
    timer::Timer,
};

/// This type is a bit bogus, PB10 is the only one that matters.
type BuzzerPwmPins = (
    Option<PA0<Alternate<AF2>>>,
    Option<PA1<Alternate<AF2>>>,
    Option<PB10<Alternate<AF2>>>,
    Option<PA3<Alternate<AF2>>>,
);

/// The buzzer on the secondary board. This should not be used on the primary board.
pub struct Buzzer {
    pwm: Pwm<TIMER1, BuzzerPwmPins>,
}

impl Buzzer {
    pub(super) fn new(
        timer1: TIMER1,
        buzzer_pin: PB10<Alternate<AF2>>,
        clocks: Clocks,
        apb1: &mut APB1,
    ) -> Self {
        let pins = (None, None, Some(buzzer_pin), None);
        let pwm = Timer::timer1(timer1, &clocks, apb1).pwm(pins, 1.khz());
        Self { pwm }
    }

    /// Set the frequency of the buzzer, or turn it off.
    pub fn set_frequency(&mut self, frequency: Option<impl Into<Hertz>>) {
        if let Some(frequency) = frequency {
            self.pwm.set_period(frequency.into());
            self.pwm.set_duty(Channel::C2, self.pwm.get_max_duty() / 2);
            self.pwm.enable(Channel::C2);
        } else {
            self.pwm.disable(Channel::C2);
        }
    }
}
