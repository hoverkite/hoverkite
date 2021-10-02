mod adc;
mod buzzer;
mod interrupts;
mod motor;
mod serial;
pub mod util;

use self::adc::AdcDmaState;
pub use self::adc::AdcReadings;
pub use self::buzzer::Buzzer;
use self::interrupts::{unmask_interrupts, SHARED};
use self::motor::{HallSensors, Motor};
use self::serial::{setup_usart0_buffered_writer, setup_usart1_buffered_writer};
use self::util::buffered_tx::BufferedSerialWriter;
use crate::log;
use bmi160::{
    interface::I2cInterface, AccelerometerPowerMode, Bmi160, GyroscopePowerMode, SlaveAddr,
};
use cortex_m::interrupt::free;
use gd32f1x0_hal::{
    gpio::{
        gpioa::{PA0, PA12, PA15},
        gpiob::{PB2, PB3, PB8, PB9},
        gpioc::PC15,
        gpiof::PF0,
        Alternate, Floating, Input, Output, OutputMode, PullMode, PullUp, PushPull, AF1,
    },
    i2c::{BlockingI2c, Mode},
    pac::{ADC, DMA, DWT, GPIOA, GPIOB, GPIOC, GPIOF, I2C0, TIMER0, TIMER1, USART0, USART1},
    prelude::*,
    pwm::Channel,
    rcu::{Clocks, AHB, APB1, APB2},
    serial::{Config, Rx, Serial, Tx},
};

const USART_BAUD_RATE: u32 = 115200;
const MOTOR_PWM_FREQ_HERTZ: u32 = 16000;

// These settings are fairly arbitrary, but they seem to work.
const I2C_FREQUENCY_HERTZ: u32 = 400_000;
const I2C_START_TIMEOUT_US: u32 = 1000;
const I2C_START_RETRIES: u8 = 3;
const I2C_ADDR_TIMEOUT_US: u32 = 1000;
const I2C_DATA_TIMEOUT_US: u32 = 1000;

pub struct Leds {
    pub side: PA0<Output<PushPull>>,
    pub green: PA15<Output<PushPull>>,
    pub orange: PA12<Output<PushPull>>,
    pub red: PB3<Output<PushPull>>,
}

pub struct Hoverboard {
    pub serial_remote_rx: Rx<USART0>,
    pub serial_remote_writer: BufferedSerialWriter<Tx<USART0>>,
    pub serial_rx: Rx<USART1>,
    pub serial_writer: BufferedSerialWriter<Tx<USART1>>,
    pub imu: Bmi160<I2cInterface<BlockingI2c<I2C0, PB8<Alternate<AF1>>, PB9<Alternate<AF1>>>>>,
    pub buzzer: Buzzer,
    pub power_latch: PB2<Output<PushPull>>,
    /// This will be high when the power button is pressed.
    pub power_button: PC15<Input<Floating>>,
    /// This will be low when the charger is connected.
    pub charge_state: PF0<Input<PullUp>>,
    pub leds: Leds,
    /// Whether to negate the motor position readings and power setting.
    pub negate_motor: bool,
}

impl Hoverboard {
    pub fn new(
        gpioa: GPIOA,
        gpiob: GPIOB,
        gpioc: GPIOC,
        gpiof: GPIOF,
        usart0: USART0,
        usart1: USART1,
        i2c0: I2C0,
        timer0: TIMER0,
        timer1: TIMER1,
        dma: DMA,
        adc: ADC,
        ahb: &mut AHB,
        apb1: &mut APB1,
        apb2: &mut APB2,
        dwt: &mut DWT,
        clocks: Clocks,
        negate_motor: bool,
    ) -> Hoverboard {
        let mut gpioa = gpioa.split(ahb);
        let mut gpiob = gpiob.split(ahb);
        let mut gpioc = gpioc.split(ahb);
        let mut gpiof = gpiof.split(ahb);

        // USART0
        let tx0 =
            gpiob
                .pb6
                .into_alternate(&mut gpiob.config, PullMode::Floating, OutputMode::PushPull);
        let rx0 =
            gpiob
                .pb7
                .into_alternate(&mut gpiob.config, PullMode::Floating, OutputMode::PushPull);
        let (serial_remote_tx, serial_remote_rx) = Serial::usart(
            usart0,
            (tx0, rx0),
            Config {
                baudrate: USART_BAUD_RATE.bps(),
                ..Config::default()
            },
            clocks,
            apb2,
        )
        .split();
        let serial_remote_writer = setup_usart0_buffered_writer(serial_remote_tx);

        // USART1
        let tx1 =
            gpioa
                .pa2
                .into_alternate(&mut gpioa.config, PullMode::Floating, OutputMode::PushPull);
        let rx1 =
            gpioa
                .pa3
                .into_alternate(&mut gpioa.config, PullMode::Floating, OutputMode::PushPull);
        let (serial_tx, serial_rx) = Serial::usart(
            usart1,
            (tx1, rx1),
            Config {
                baudrate: USART_BAUD_RATE.bps(),
                ..Config::default()
            },
            clocks,
            apb1,
        )
        .split();
        let mut serial_writer = setup_usart1_buffered_writer(serial_tx);

        // I2C0
        let scl =
            gpiob
                .pb8
                .into_alternate(&mut gpiob.config, PullMode::Floating, OutputMode::OpenDrain);
        let sda =
            gpiob
                .pb9
                .into_alternate(&mut gpiob.config, PullMode::Floating, OutputMode::OpenDrain);
        dwt.enable_cycle_counter();
        let i2c = BlockingI2c::i2c0(
            i2c0,
            scl,
            sda,
            Mode::Standard {
                frequency: I2C_FREQUENCY_HERTZ.hz(),
            },
            clocks,
            apb1,
            I2C_START_TIMEOUT_US,
            I2C_START_RETRIES,
            I2C_ADDR_TIMEOUT_US,
            I2C_DATA_TIMEOUT_US,
        );
        // It's actually a BMI120 (or a clone of it), but the BMI160 is close enough that it works.
        let mut imu = Bmi160::new_with_i2c(i2c, SlaveAddr::Default);
        match imu.chip_id() {
            Err(e) => log!(&mut serial_writer, "Error reading chip ID: {:?}", e),
            Ok(chip_id) => log!(&mut serial_writer, "Chip ID: {:#x}", chip_id),
        }
        if let Err(e) = imu.set_accel_power_mode(AccelerometerPowerMode::Normal) {
            log!(
                &mut serial_writer,
                "Error setting accelerometer power mode: {:?}",
                e
            );
        }
        if let Err(e) = imu.set_gyro_power_mode(GyroscopePowerMode::Normal) {
            log!(
                &mut serial_writer,
                "Error setting gyroscope power mode: {:?}",
                e
            );
        }

        // DMA controller
        let dma = dma.split(ahb);

        // ADC
        let battery_voltage = gpioa.pa4.into_analog(&mut gpioa.config);
        let motor_current = gpioa.pa6.into_analog(&mut gpioa.config);
        let adc_dma = AdcDmaState::setup(adc, battery_voltage, motor_current, apb2, clocks, dma.0);

        // Motor
        // Output speed defaults to 2MHz
        let green_high =
            gpioa
                .pa10
                .into_alternate(&mut gpioa.config, PullMode::Floating, OutputMode::PushPull);
        let blue_high =
            gpioa
                .pa9
                .into_alternate(&mut gpioa.config, PullMode::Floating, OutputMode::PushPull);
        let yellow_high =
            gpioa
                .pa8
                .into_alternate(&mut gpioa.config, PullMode::Floating, OutputMode::PushPull);
        let green_low =
            gpiob
                .pb15
                .into_alternate(&mut gpiob.config, PullMode::Floating, OutputMode::PushPull);
        let blue_low =
            gpiob
                .pb14
                .into_alternate(&mut gpiob.config, PullMode::Floating, OutputMode::PushPull);
        let yellow_low =
            gpiob
                .pb13
                .into_alternate(&mut gpiob.config, PullMode::Floating, OutputMode::PushPull);
        let emergency_off =
            gpiob
                .pb12
                .into_alternate(&mut gpiob.config, PullMode::Floating, OutputMode::PushPull);
        let motor_pins = (
            (yellow_high, yellow_low),
            (blue_high, blue_low),
            (green_high, green_low),
        );
        let hall_sensors = HallSensors::new(
            gpiob.pb11.into_floating_input(&mut gpiob.config),
            gpiof.pf1.into_floating_input(&mut gpiof.config),
            gpioc.pc14.into_floating_input(&mut gpioc.config),
        );
        let motor = Motor::new(
            timer0,
            MOTOR_PWM_FREQ_HERTZ.hz(),
            clocks,
            motor_pins,
            emergency_off,
            apb2,
            hall_sensors,
        );

        // Buzzer
        let buzzer_pin =
            gpiob
                .pb10
                .into_alternate(&mut gpiob.config, PullMode::Floating, OutputMode::PushPull);
        let buzzer = Buzzer::new(timer1, buzzer_pin, clocks, apb1);

        unmask_interrupts(motor, adc_dma);

        Hoverboard {
            serial_remote_rx,
            serial_remote_writer,
            serial_rx,
            serial_writer,
            imu,
            buzzer,
            power_latch: gpiob.pb2.into_push_pull_output(&mut gpiob.config),
            power_button: gpioc.pc15.into_floating_input(&mut gpioc.config),
            charge_state: gpiof.pf0.into_pull_up_input(&mut gpiof.config),
            leds: Leds {
                side: gpioa.pa0.into_push_pull_output(&mut gpioa.config),
                green: gpioa.pa15.into_push_pull_output(&mut gpioa.config),
                orange: gpioa.pa12.into_push_pull_output(&mut gpioa.config),
                red: gpiob.pb3.into_push_pull_output(&mut gpiob.config),
            },
            negate_motor,
        }
    }

    pub fn adc_readings(&self) -> AdcReadings {
        free(|cs| {
            if let Some(shared) = &mut *SHARED.borrow(cs).borrow_mut() {
                shared.last_adc_readings.clone()
            } else {
                AdcReadings::default()
            }
        })
    }

    /// Get the current position of the motor.
    pub fn motor_position(&self) -> i64 {
        free(|cs| {
            // SHARED must have been initialised by the time this is called.
            let shared = &mut *SHARED.borrow(cs).borrow_mut();
            let shared = shared.as_mut().unwrap();

            if self.negate_motor {
                -shared.motor.position
            } else {
                shared.motor.position
            }
        })
    }

    /// Set the desired power for the motor.
    pub fn set_motor_power(&mut self, power: i16) {
        free(|cs| {
            // SHARED must have been initialised by the time this is called.
            let shared = &mut *SHARED.borrow(cs).borrow_mut();
            let shared = shared.as_mut().unwrap();

            shared.motor.target_power = if self.negate_motor { -power } else { power };
        })
    }

    /// Set the motor's current position as 0.
    pub fn recenter_motor(&mut self) {
        free(|cs| {
            // SHARED must have been initialised by the time this is called.
            let shared = &mut *SHARED.borrow(cs).borrow_mut();
            let shared = shared.as_mut().unwrap();

            shared.motor.position = 0;
        })
    }

    /// Set the motor PWM values directly for testing.
    pub fn set_motor_pwm_for_test(&mut self, y_percent: u8, b_percent: u8, g_percent: u8) {
        free(|cs| {
            // SHARED must have been initialised by the time this is called.
            let shared = &mut *SHARED.borrow(cs).borrow_mut();
            let shared = shared.as_mut().unwrap();
            let pwm = &mut shared.motor.pwm;

            let duty_max = pwm.get_max_duty() as u32;
            pwm.set_duty(Channel::C0, (duty_max * y_percent as u32 / 100) as u16);
            pwm.set_duty(Channel::C1, (duty_max * b_percent as u32 / 100) as u16);
            pwm.set_duty(Channel::C2, (duty_max * g_percent as u32 / 100) as u16);
            pwm.automatic_output_enable();
        })
    }
}
