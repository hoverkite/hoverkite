use crate::pwm::Pwm;
use crate::util::clamp;
use gd32f1x0_hal::{
    gpio::{
        gpioa::{PA10, PA8, PA9},
        gpiob::{PB11, PB12, PB13, PB14, PB15},
        gpioc::PC14,
        gpiof::PF1,
        Alternate, Floating, Input, AF2,
    },
    prelude::*,
};

/// The minimum number of timer interrupt cycles to wait between increasing the motor power by one
/// step.
const MOTOR_POWER_SMOOTHING_CYCLES_PER_STEP: u32 = 5;

/// If the motor power is below this level, don't bother running it at all.
const MOTOR_POWER_DEAD_ZONE: i16 = 10;

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
    #[allow(dead_code)]
    green_high: PA10<Alternate<AF2>>,
    #[allow(dead_code)]
    blue_high: PA9<Alternate<AF2>>,
    #[allow(dead_code)]
    yellow_high: PA8<Alternate<AF2>>,
    #[allow(dead_code)]
    green_low: PB15<Alternate<AF2>>,
    #[allow(dead_code)]
    blue_low: PB14<Alternate<AF2>>,
    #[allow(dead_code)]
    yellow_low: PB13<Alternate<AF2>>,
    #[allow(dead_code)]
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
        // If power is below a threshold, turn it off entirely.
        if power.abs() < MOTOR_POWER_DEAD_ZONE {
            self.pwm.set_duty_cycles(0, 0, 0);
            return;
        }

        let power: i16 = clamp(power, &(-1000..=1000));
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
        let y = clamp((y + power_max) as u16, &(10..=duty_max - 10));
        let b = clamp((b + power_max) as u16, &(10..=duty_max - 10));
        let g = clamp((g + power_max) as u16, &(10..=duty_max - 10));
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
            // MOTOR_POWER_SMOOTHING_CYCLES_PER_STEP interrupts. This is only applied when
            // increasing the power, not decreasing, to avoid overshooting.
            if self.smoothing_cycles < MOTOR_POWER_SMOOTHING_CYCLES_PER_STEP
                && self.target_power.abs() > self.power.abs()
            {
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
