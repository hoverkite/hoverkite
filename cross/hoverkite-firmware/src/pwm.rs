use gd32f1x0_hal::{
    gpio::{
        gpioa::{PA10, PA8, PA9},
        gpiob::{PB12, PB13, PB14, PB15},
        Alternate, AF2,
    },
    pac::TIMER0,
    prelude::*,
    pwm::{Alignment, BreakMode, Channel, IdleState, Polarity, Pwm as HalPwm},
    rcu::{Clocks, APB2},
    time::Hertz,
    timer::{Event, Timer},
};

pub struct Pwm {
    pub pwm: HalPwm<TIMER0, OptionalPins>,
    _emergency_off: PB12<Alternate<AF2>>,
}

pub type Pins = (
    (PA8<Alternate<AF2>>, PB13<Alternate<AF2>>),
    (PA9<Alternate<AF2>>, PB14<Alternate<AF2>>),
    (PA10<Alternate<AF2>>, PB15<Alternate<AF2>>),
);

type OptionalPins = (
    Option<(PA8<Alternate<AF2>>, PB13<Alternate<AF2>>)>,
    Option<(PA9<Alternate<AF2>>, PB14<Alternate<AF2>>)>,
    Option<(PA10<Alternate<AF2>>, PB15<Alternate<AF2>>)>,
);

impl Pwm {
    pub fn new(
        timer: TIMER0,
        frequency: Hertz,
        clocks: Clocks,
        pins: Pins,
        emergency_off: PB12<Alternate<AF2>>,
        apb: &mut APB2,
    ) -> Self {
        let pins = (Some(pins.0), Some(pins.1), Some(pins.2));
        let mut pwm = Timer::timer0(timer, &clocks, apb).pwm(pins, frequency);

        pwm.set_alignment(Alignment::Center);

        let channels = [Channel::C0, Channel::C1, Channel::C2];
        // Configure output channels and set duty cycle to 0 on all channels.
        for channel in &channels {
            pwm.set_duty(*channel, 0);
            pwm.set_polarity(*channel, Polarity::NotInverted);
            pwm.set_complementary_polarity(*channel, Polarity::Inverted);
            pwm.set_idle_state(*channel, IdleState::Low);
            pwm.set_complementary_idle_state(*channel, IdleState::High);
        }

        // Configure break parameters
        pwm.set_dead_time(60);
        pwm.break_enable(BreakMode::ActiveLow);
        pwm.run_mode_off_state(true);
        pwm.idle_mode_off_state(false);

        // Enable PWM output on all channels and complementary channels.
        for channel in &channels {
            pwm.enable(*channel);
        }

        // Enable timer interrupt
        // TODO: Set priority?
        pwm.listen(Event::Update);

        Pwm {
            pwm,
            _emergency_off: emergency_off,
        }
    }

    pub fn set_duty_cycles(&mut self, y: u16, b: u16, g: u16) {
        self.pwm.set_duty(Channel::C0, y);
        self.pwm.set_duty(Channel::C1, b);
        self.pwm.set_duty(Channel::C2, g);
    }
}
