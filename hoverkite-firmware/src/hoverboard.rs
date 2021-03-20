use stm32f0xx_hal::{
    gpio::{
        gpioa::{PA0, PA12, PA15, PA2, PA3},
        gpiob::{PB10, PB3},
        Alternate, Output, PushPull, AF1,
    },
    pac::{GPIOA, GPIOB, USART2},
    prelude::*,
    rcc::Rcc,
    serial::Serial,
};

const USART_BAUD_RATE: u32 = 115200;

pub struct Hoverboard {
    pub serial: Serial<USART2, PA2<Alternate<AF1>>, PA3<Alternate<AF1>>>,
    pub side_led: PA0<Output<PushPull>>,
    pub green_led: PA15<Output<PushPull>>,
    pub orange_led: PA12<Output<PushPull>>,
    pub red_led: PB3<Output<PushPull>>,
    pub buzzer: PB10<Output<PushPull>>,
}

impl Hoverboard {
    pub fn new(gpioa: GPIOA, gpiob: GPIOB, usart2: USART2, rcc: &mut Rcc) -> Hoverboard {
        let gpioa = gpioa.split(rcc);
        let gpiob = gpiob.split(rcc);

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

        // USART
        let pa2 = gpioa.pa2;
        let pa3 = gpioa.pa3;
        let (tx, rx) = cortex_m::interrupt::free(move |cs| {
            (pa2.into_alternate_af1(cs), pa3.into_alternate_af1(cs))
        });
        let serial = Serial::usart2(usart2, (tx, rx), USART_BAUD_RATE.bps(), rcc);

        Hoverboard {
            serial,
            side_led,
            green_led,
            orange_led,
            red_led,
            buzzer,
        }
    }
}
