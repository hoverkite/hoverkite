use gd32f1x0_hal::{
    gpio::{
        gpioa::{PA0, PA10, PA12, PA15, PA2, PA3, PA4, PA6, PA8, PA9},
        gpiob::{PB10, PB11, PB12, PB13, PB14, PB15, PB2, PB3},
        gpioc::{PC14, PC15},
        gpiof::{PF0, PF1},
        Alternate, Analog, Floating, Input, Output, OutputMode, PullMode, PullUp, PushPull, AF1,
        AF2,
    },
    pac::{GPIOA, GPIOB, GPIOC, GPIOF, USART1},
    prelude::*,
    rcu::{Clocks, AHB, APB1},
    serial::{Config, Serial},
};

const USART_BAUD_RATE: u32 = 115200;

pub struct HallSensors {
    hall_a: PB11<Input<Floating>>,
    hall_b: PF1<Input<Floating>>,
    hall_c: PC14<Input<Floating>>,
}

impl HallSensors {
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

pub struct Leds {
    pub side: PA0<Output<PushPull>>,
    pub green: PA15<Output<PushPull>>,
    pub orange: PA12<Output<PushPull>>,
    pub red: PB3<Output<PushPull>>,
}

pub struct Motor {
    pub green_high: PA10<Alternate<AF2>>,
    pub blue_high: PA9<Alternate<AF2>>,
    pub yellow_high: PA8<Alternate<AF2>>,
    pub green_low: PB15<Alternate<AF2>>,
    pub blue_low: PB14<Alternate<AF2>>,
    pub yellow_low: PB13<Alternate<AF2>>,
    pub emergency_off: PB12<Alternate<AF2>>,
}

pub struct Hoverboard {
    pub serial: Serial<USART1, PA2<Alternate<AF1>>, PA3<Alternate<AF1>>>,
    pub buzzer: PB10<Output<PushPull>>,
    pub power_latch: PB2<Output<PushPull>>,
    pub power_button: PC15<Input<Floating>>,
    /// This will be low when the charger is connected.
    pub charge_state: PF0<Input<PullUp>>,
    pub battery_voltage: PA4<Analog>,
    pub current: PA6<Analog>,
    pub leds: Leds,
    pub hall_sensors: HallSensors,
    pub motor: Motor,
}

impl Hoverboard {
    pub fn new(
        gpioa: GPIOA,
        gpiob: GPIOB,
        gpioc: GPIOC,
        gpiof: GPIOF,
        usart1: USART1,
        ahb: &mut AHB,
        apb1: &mut APB1,
        clocks: Clocks,
    ) -> Hoverboard {
        let mut gpioa = gpioa.split(ahb);
        let mut gpiob = gpiob.split(ahb);
        let mut gpioc = gpioc.split(ahb);
        let mut gpiof = gpiof.split(ahb);

        // NB: Don't try to use pa13, that's SWDIO

        // USART
        let tx =
            gpioa
                .pa2
                .into_alternate(&mut gpioa.config, PullMode::Floating, OutputMode::PushPull);
        let rx =
            gpioa
                .pa3
                .into_alternate(&mut gpioa.config, PullMode::Floating, OutputMode::PushPull);

        Hoverboard {
            serial: Serial::usart(
                usart1,
                (tx, rx),
                Config {
                    baudrate: USART_BAUD_RATE.bps(),
                    ..Config::default()
                },
                clocks,
                apb1,
            ),
            buzzer: gpiob.pb10.into_push_pull_output(&mut gpiob.config),
            power_latch: gpiob.pb2.into_push_pull_output(&mut gpiob.config),
            power_button: gpioc.pc15.into_floating_input(&mut gpioc.config),
            charge_state: gpiof.pf0.into_pull_up_input(&mut gpiof.config),
            battery_voltage: gpioa.pa4.into_analog(&mut gpioa.config),
            current: gpioa.pa6.into_analog(&mut gpioa.config),
            leds: Leds {
                side: gpioa.pa0.into_push_pull_output(&mut gpioa.config),
                green: gpioa.pa15.into_push_pull_output(&mut gpioa.config),
                orange: gpioa.pa12.into_push_pull_output(&mut gpioa.config),
                red: gpiob.pb3.into_push_pull_output(&mut gpiob.config),
            },
            hall_sensors: HallSensors {
                hall_a: gpiob.pb11.into_floating_input(&mut gpiob.config),
                hall_b: gpiof.pf1.into_floating_input(&mut gpiof.config),
                hall_c: gpioc.pc14.into_floating_input(&mut gpioc.config),
            },
            motor: Motor {
                // Output speed defaults to 2MHz
                green_high: gpioa.pa10.into_alternate(
                    &mut gpioa.config,
                    PullMode::Floating,
                    OutputMode::PushPull,
                ),
                blue_high: gpioa.pa9.into_alternate(
                    &mut gpioa.config,
                    PullMode::Floating,
                    OutputMode::PushPull,
                ),
                yellow_high: gpioa.pa8.into_alternate(
                    &mut gpioa.config,
                    PullMode::Floating,
                    OutputMode::PushPull,
                ),
                green_low: gpiob.pb15.into_alternate(
                    &mut gpiob.config,
                    PullMode::Floating,
                    OutputMode::PushPull,
                ),
                blue_low: gpiob.pb14.into_alternate(
                    &mut gpiob.config,
                    PullMode::Floating,
                    OutputMode::PushPull,
                ),
                yellow_low: gpiob.pb13.into_alternate(
                    &mut gpiob.config,
                    PullMode::Floating,
                    OutputMode::PushPull,
                ),
                emergency_off: gpiob.pb12.into_alternate(
                    &mut gpiob.config,
                    PullMode::Floating,
                    OutputMode::PushPull,
                ),
            },
        }
    }
}
