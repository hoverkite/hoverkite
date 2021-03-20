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

pub struct Hoverboard {
    pub serial: Serial<USART2, PA2<Alternate<AF1>>, PA3<Alternate<AF1>>>,
    pub side_led: PA0<Output<PushPull>>,
    pub green_led: PA15<Output<PushPull>>,
    pub orange_led: PA12<Output<PushPull>>,
    pub red_led: PB3<Output<PushPull>>,
    pub buzzer: PB10<Output<PushPull>>,
    pub power_latch: PB2<Output<PushPull>>,
    pub power_button: PC15<Input<Floating>>,
    pub charge_state: PF0<Input<PullUp>>,
    pub battery_voltage: PA4<Analog>,
    pub current: PA6<Analog>,
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

        // LEDs
        let pa0 = gpioa.pa0;
        let pa15 = gpioa.pa15;
        let pa12 = gpioa.pa12;
        let pb3 = gpiob.pb3;
        let (side_led, green_led, orange_led, red_led) = cortex_m::interrupt::free(|cs| {
            (
                pa0.into_push_pull_output(cs),
                pa15.into_push_pull_output(cs),
                pa12.into_push_pull_output(cs),
                pb3.into_push_pull_output(cs),
            )
        });

        // Buzzer
        let pb10 = gpiob.pb10;
        let buzzer = cortex_m::interrupt::free(|cs| pb10.into_push_pull_output(cs));

        // Hall effect sensors
        let pb11 = gpiob.pb11;
        let pf1 = gpiof.pf1;
        let pc14 = gpioc.pc14;
        let (hall_a, hall_b, hall_c) = cortex_m::interrupt::free(|cs| {
            (
                pb11.into_floating_input(cs),
                pf1.into_floating_input(cs),
                pc14.into_floating_input(cs),
            )
        });

        // Power latch, power button and charge state
        let pb2 = gpiob.pb2;
        let pc15 = gpioc.pc15;
        let pf0 = gpiof.pf0;
        let (power_latch, power_button, charge_state) = cortex_m::interrupt::free(|cs| {
            (
                pb2.into_push_pull_output(cs),
                pc15.into_floating_input(cs),
                pf0.into_pull_up_input(cs),
            )
        });

        // USART
        let pa2 = gpioa.pa2;
        let pa3 = gpioa.pa3;
        let (tx, rx) = cortex_m::interrupt::free(move |cs| {
            (pa2.into_alternate_af1(cs), pa3.into_alternate_af1(cs))
        });
        let serial = Serial::usart2(usart2, (tx, rx), USART_BAUD_RATE.bps(), rcc);

        // Battery voltage and current
        let pa4 = gpioa.pa4;
        let pa6 = gpioa.pa6;
        let (battery_voltage, current) =
            cortex_m::interrupt::free(|cs| (pa4.into_analog(cs), pa6.into_analog(cs)));

        Hoverboard {
            serial,
            side_led,
            green_led,
            orange_led,
            red_led,
            buzzer,
            power_latch,
            power_button,
            charge_state,
            battery_voltage,
            current,
            hall_sensors: HallSensors {
                hall_a,
                hall_b,
                hall_c,
            },
        }
    }
}
