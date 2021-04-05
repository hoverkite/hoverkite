use gd32f1x0_hal::{
    gpio::{
        gpioa::{PA10, PA8, PA9},
        gpiob::{PB11, PB12, PB13, PB14, PB15},
        gpioc::PC14,
        gpiof::PF1,
        Alternate, Floating, Input, AF2,
    },
    pac::TIMER0,
    prelude::*,
    pwm::Channel,
    rcu::{Clocks, Enable, Reset, APB2},
    time::Hertz,
};

const MOTOR_POWER_SMOOTHING_CYCLES_PER_STEP: u32 = 10;

pub struct HallSensors {
    hall_a: PB11<Input<Floating>>,
    hall_b: PF1<Input<Floating>>,
    hall_c: PC14<Input<Floating>>,
}

impl HallSensors {
    pub fn new(
        hall_a: PB11<Input<Floating>>,
        hall_b: PF1<Input<Floating>>,
        hall_c: PC14<Input<Floating>>,
    ) -> Self {
        Self {
            hall_a,
            hall_b,
            hall_c,
        }
    }

    /// Get the current position of the motor from the hall effoct sensors, or `None` if they are in
    /// an invalid configuration.
    ///
    /// The position will be in the range 0-5, inclusive.
    pub fn position(&self) -> Option<u8> {
        let hall_a = self.hall_a.is_high().unwrap();
        let hall_b = self.hall_b.is_high().unwrap();
        let hall_c = self.hall_c.is_high().unwrap();
        match (hall_a, hall_b, hall_c) {
            (false, false, true) => Some(0),
            (true, false, true) => Some(1),
            (true, false, false) => Some(2),
            (true, true, false) => Some(3),
            (false, true, false) => Some(4),
            (false, true, true) => Some(5),
            _ => None,
        }
    }
}

pub struct Motor {
    green_high: PA10<Alternate<AF2>>,
    blue_high: PA9<Alternate<AF2>>,
    yellow_high: PA8<Alternate<AF2>>,
    green_low: PB15<Alternate<AF2>>,
    blue_low: PB14<Alternate<AF2>>,
    yellow_low: PB13<Alternate<AF2>>,
    emergency_off: PB12<Alternate<AF2>>,
    pub pwm: Pwm,
    hall_sensors: HallSensors,
    /// The absolute position of the motor.
    pub position: i64,
    /// The last valid reading from the Hall sensors.
    last_hall_position: Option<u8>,
    /// The desired motor power.
    pub target_power: i16,
    /// The last set motor power.
    power: i16,
    /// The number of timer cycles since the motor power was last changed.
    smoothing_cycles: u32,
}

impl Motor {
    pub fn new(
        green_high: PA10<Alternate<AF2>>,
        blue_high: PA9<Alternate<AF2>>,
        yellow_high: PA8<Alternate<AF2>>,
        green_low: PB15<Alternate<AF2>>,
        blue_low: PB14<Alternate<AF2>>,
        yellow_low: PB13<Alternate<AF2>>,
        emergency_off: PB12<Alternate<AF2>>,
        pwm: Pwm,
        hall_sensors: HallSensors,
    ) -> Self {
        Self {
            green_high,
            blue_high,
            yellow_high,
            green_low,
            blue_low,
            yellow_low,
            emergency_off,
            pwm,
            hall_sensors,
            position: 0,
            last_hall_position: None,
            power: 0,
            target_power: 0,
            smoothing_cycles: 0,
        }
    }

    fn set_position_power(&mut self, power: i16, position: u8) {
        let power = clamp(power, -1000, 1000);
        // TODO: Low-pass filter power
        let (y, b, g) = match position {
            0 => (0, -power, power),
            1 => (power, -power, 0),
            2 => (power, 0, -power),
            3 => (0, power, -power),
            4 => (-power, power, 0),
            5 => (-power, 0, power),
            _ => (0, 0, 0),
        };
        let duty_max = self.pwm.duty_max();
        let power_max = (duty_max / 2) as i32;
        let y = y as i32 * power_max / 1000;
        let b = b as i32 * power_max / 1000;
        let g = g as i32 * power_max / 1000;
        let y = clamp((y + power_max) as u16, 10, duty_max - 10);
        let b = clamp((b + power_max) as u16, 10, duty_max - 10);
        let g = clamp((g + power_max) as u16, 10, duty_max - 10);
        self.pwm.set_duty_cycles(y, b, g);
    }

    /// This should be called at regular intervals from the timer interrupt.
    pub fn update(&mut self) {
        // Read the Hall effect sensors on the motor.
        if let Some(hall_position) = self.hall_sensors.position() {
            if let Some(last_hall_position) = self.last_hall_position {
                // Update absolute position.
                let difference = (6 + hall_position - last_hall_position) % 6;
                match difference {
                    0 => {}
                    1 => self.position += 1,
                    2 => self.position += 2,
                    4 => self.position -= 2,
                    5 => self.position -= 1,
                    _ => {
                        // TODO: Log error
                    }
                }
            }

            self.last_hall_position = Some(hall_position);

            // Smoothing for motor power: don't change more than one unit every
            // MOTOR_POWER_SMOOTHING_CYCLES_PER_STEP interrupts.
            if self.smoothing_cycles < MOTOR_POWER_SMOOTHING_CYCLES_PER_STEP {
                self.smoothing_cycles += 1;
            } else if self.target_power > self.power {
                self.power += 1;
                self.smoothing_cycles = 0;
            } else if self.target_power < self.power {
                self.power -= 1;
                self.smoothing_cycles = 0;
            }

            // Set motor position based on desired power and Hall sensor reading.
            self.set_position_power(self.power, hall_position);
        }
    }
}

pub struct Pwm {
    pub timer: TIMER0,
    auto_reload_value: u16,
}

impl Pwm {
    pub fn new(timer: TIMER0, frequency: Hertz, clocks: Clocks, apb: &mut APB2) -> Self {
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
        // TODO: Can this just be a bit shift? Why the '- 1'?
        let prescaler = ((period - 1) / (1 << 16)) as u16;
        let auto_reload_value = (period / (prescaler + 1) as u32) as u16;
        timer.psc.write(|w| w.psc().bits(prescaler));
        timer.car.write(|w| w.car().bits(auto_reload_value));

        // Configure repetition counter.
        timer.crep.write(|w| unsafe { w.crep().bits(0) });

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
            .modify(|_, w| w.ch0comctl().pwm_mode2().ch1comctl().pwm_mode2());
        timer
            .chctl1_output()
            .modify(|_, w| w.ch2comctl().pwm_mode2());

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
        }
    }

    pub fn automatic_output_disable(&mut self) {
        self.timer.cchp.modify(|_, w| w.oaen().manual());
    }

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

fn clamp<T: PartialOrd>(x: T, low: T, high: T) -> T {
    if x > high {
        high
    } else if x < low {
        low
    } else {
        x
    }
}
