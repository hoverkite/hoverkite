use gd32f1x0_hal::{
    gpio::{
        gpioa::{PA10, PA8, PA9},
        gpiob::{PB12, PB13, PB14, PB15},
        Alternate, AF2,
    },
    pac::TIMER0,
    pwm::Channel,
    rcu::{Clocks, Enable, Reset, APB2},
    time::Hertz,
};

pub struct Pwm {
    pub timer: TIMER0,
    auto_reload_value: u16,
    _pins: Pins,
    _emergency_off: PB12<Alternate<AF2>>,
}

pub type Pins = (
    (PA8<Alternate<AF2>>, PB13<Alternate<AF2>>),
    (PA9<Alternate<AF2>>, PB14<Alternate<AF2>>),
    (PA10<Alternate<AF2>>, PB15<Alternate<AF2>>),
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
        // Enable clock
        TIMER0::enable(apb);

        // Reset timer via RCU
        TIMER0::reset(apb);

        // Configure direction, aligned mode and clock division. Disable auto-reload shadow.
        timer.ctl0.modify(|_, w| {
            w.dir()
                .up()
                .cam()
                .center_aligned_counting_up()
                .ckdiv()
                .div1()
                .arse()
                .disabled()
        });

        // Configure prescaler and auto-reload value to give desired period.
        // If pclk is prescaled from hclk, the frequency fed into the timers is doubled
        let tclk = clocks.pclk2_tim().0;
        let period = tclk / frequency.0;
        let prescaler = ((period - 1) >> 16) as u16;
        let auto_reload_value = (period / (prescaler + 1) as u32) as u16;
        timer.psc.write(|w| w.psc().bits(prescaler));
        timer.car.write(|w| w.car().bits(auto_reload_value));

        // Configure repetition counter.
        timer.crep.write(|w| w.crep().bits(0));

        let channels = [Channel::C0, Channel::C1, Channel::C2];
        for channel in &channels {
            // Set duty cycle to 0.
            match channel {
                Channel::C0 => timer.ch0cv.write(|w| w.ch0val().bits(0)),
                Channel::C1 => timer.ch1cv.write(|w| w.ch1val().bits(0)),
                Channel::C2 => timer.ch2cv.write(|w| w.ch2val().bits(0)),
                Channel::C3 => timer.ch3cv.write(|w| w.ch3val().bits(0)),
            }
        }
        // Deactivate fastmode for all channels.
        timer
            .chctl0_output()
            .modify(|_, w| w.ch0comfen().slow().ch1comfen().slow());
        timer.chctl1_output().modify(|_, w| w.ch2comfen().slow());
        // Deactivate output shadow function for all channels.
        timer
            .chctl0_output()
            .modify(|_, w| w.ch0comsen().disabled().ch1comsen().disabled());
        timer
            .chctl1_output()
            .modify(|_, w| w.ch2comsen().disabled());
        // Set all output channel PWM types to PWM1
        timer
            .chctl0_output()
            .modify(|_, w| w.ch0comctl().pwm_mode1().ch1comctl().pwm_mode1());
        timer
            .chctl1_output()
            .modify(|_, w| w.ch2comctl().pwm_mode1());

        // Configure output channels
        timer.chctl2.modify(|_, w| {
            w.ch0p()
                .not_inverted()
                .ch0np()
                .inverted()
                .ch1p()
                .not_inverted()
                .ch1np()
                .inverted()
                .ch2p()
                .not_inverted()
                .ch2np()
                .inverted()
        });
        timer.ctl1.modify(|_, w| {
            w.iso0()
                .low()
                .iso0n()
                .high()
                .iso1()
                .low()
                .iso1n()
                .high()
                .iso2()
                .low()
                .iso2n()
                .high()
        });

        // Configure break parameters
        timer.cchp.write(|w| {
            w.ros()
                .enabled()
                .ios()
                .disabled()
                .prot()
                .disabled()
                .dtcfg()
                .bits(60)
                .brken()
                .enabled()
                .brkp()
                .inverted()
                .oaen()
                .automatic()
        });

        // Disable timer
        timer.ctl0.modify(|_, w| w.cen().disabled());

        // Enable PWM output on all channels and complementary channels.
        timer.chctl2.modify(|_, w| {
            w.ch0en()
                .enabled()
                .ch1en()
                .enabled()
                .ch2en()
                .enabled()
                .ch0nen()
                .enabled()
                .ch1nen()
                .enabled()
                .ch2nen()
                .enabled()
        });

        // Enable timer interrupt
        // TODO: Set priority?
        timer.dmainten.modify(|_, w| w.upie().enabled());

        // Enable timer
        timer.ctl0.modify(|_, w| w.cen().enabled());

        Pwm {
            timer,
            auto_reload_value,
            _pins: pins,
            _emergency_off: emergency_off,
        }
    }

    #[allow(dead_code)]
    pub fn automatic_output_disable(&mut self) {
        self.timer.cchp.modify(|_, w| w.oaen().manual());
    }

    #[allow(dead_code)]
    pub fn automatic_output_enable(&mut self) {
        self.timer.cchp.modify(|_, w| w.oaen().automatic());
    }

    pub fn set_duty_cycles(&mut self, y: u16, b: u16, g: u16) {
        self.timer.ch0cv.write(|w| w.ch0val().bits(y));
        self.timer.ch1cv.write(|w| w.ch1val().bits(b));
        self.timer.ch2cv.write(|w| w.ch2val().bits(g));
    }

    pub fn duty_max(&self) -> u16 {
        self.auto_reload_value
    }
}
