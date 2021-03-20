use stm32f0xx_hal::{
    gpio::{
        gpioa::{PA0, PA12, PA15, PA2, PA3, PA4, PA6},
        gpiob::{PB10, PB11, PB2, PB3},
        gpioc::{PC14, PC15},
        gpiof::{PF0, PF1},
        Alternate, Analog, Floating, Input, Output, PullUp, PushPull, AF1,
    },
    pac::{GPIOA, GPIOB, GPIOC, GPIOF, USART2},
    prelude::*,
    rcc::Rcc,
    serial::Serial,
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

pub struct Hoverboard {
    pub serial: Serial<USART2, PA2<Alternate<AF1>>, PA3<Alternate<AF1>>>,
    pub buzzer: PB10<Output<PushPull>>,
    pub power_latch: PB2<Output<PushPull>>,
    pub power_button: PC15<Input<Floating>>,
    /// This will be low when the charger is connected.
    pub charge_state: PF0<Input<PullUp>>,
    pub battery_voltage: PA4<Analog>,
    pub current: PA6<Analog>,
    pub leds: Leds,
    pub hall_sensors: HallSensors,
}

impl Hoverboard {
    pub fn new(
        gpioa: GPIOA,
        gpiob: GPIOB,
        gpioc: GPIOC,
        gpiof: GPIOF,
        usart2: USART2,
        rcc: &mut Rcc,
    ) -> Hoverboard {
        let gpioa = gpioa.split(rcc);
        let gpiob = gpiob.split(rcc);
        let gpioc = gpioc.split(rcc);
        let gpiof = gpiof.split(rcc);

        // NB: Don't try to use pa13, that's SWDIO

        cortex_m::interrupt::free(|cs| {
            // USART
            let tx = gpioa.pa2.into_alternate_af1(cs);
            let rx = gpioa.pa3.into_alternate_af1(cs);

            Hoverboard {
                serial: Serial::usart2(usart2, (tx, rx), USART_BAUD_RATE.bps(), rcc),
                buzzer: gpiob.pb10.into_push_pull_output(cs),
                power_latch: gpiob.pb2.into_push_pull_output(cs),
                power_button: gpioc.pc15.into_floating_input(cs),
                charge_state: gpiof.pf0.into_pull_up_input(cs),
                battery_voltage: gpioa.pa4.into_analog(cs),
                current: gpioa.pa6.into_analog(cs),
                leds: Leds {
                    side: gpioa.pa0.into_push_pull_output(cs),
                    green: gpioa.pa15.into_push_pull_output(cs),
                    orange: gpioa.pa12.into_push_pull_output(cs),
                    red: gpiob.pb3.into_push_pull_output(cs),
                },
                hall_sensors: HallSensors {
                    hall_a: gpiob.pb11.into_floating_input(cs),
                    hall_b: gpiof.pf1.into_floating_input(cs),
                    hall_c: gpioc.pc14.into_floating_input(cs),
                },
            }
        })
    }
}
