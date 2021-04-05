use core::{cell::RefCell, mem};
use cortex_m::{
    interrupt::{free, Mutex},
    peripheral::NVIC,
    singleton,
};
use gd32f1x0_hal::{
    adc::{Adc, AdcDma, SampleTime, Scan, Sequence, VBat},
    dma::{Event, Transfer, W},
    gpio::{
        gpioa::{PA0, PA10, PA12, PA15, PA2, PA3, PA8, PA9},
        gpiob::{PB10, PB11, PB12, PB13, PB14, PB15, PB2, PB3},
        gpioc::{PC14, PC15},
        gpiof::{PF0, PF1},
        Alternate, Floating, Input, Output, OutputMode, PullMode, PullUp, PushPull, AF1, AF2,
    },
    pac::{
        adc::ctl1::CTN_A, interrupt, Interrupt, ADC, DMA, GPIOA, GPIOB, GPIOC, GPIOF, TIMER0,
        USART1,
    },
    prelude::*,
    pwm::Channel,
    rcu::{Clocks, Enable, Reset, AHB, APB1, APB2},
    serial::{Config, Serial},
    time::Hertz,
};

const USART_BAUD_RATE: u32 = 115200;
const MOTOR_PWM_FREQ_HERTZ: u32 = 16000;
const CURRENT_OFFSET_DC: u16 = 1073;

struct Shared {
    pwm: Pwm,
    adc_dma: AdcDmaState,
    last_adc_readings: AdcReadings,
}

#[derive(Debug, Default, Clone)]
pub struct AdcReadings {
    pub battery_voltage: u16,
    pub motor_current: u16,
    pub backup_battery_voltage: u16,
}

impl AdcReadings {
    fn update_from_buffer(&mut self, buffer: &[u16; 3], adc: &Adc) {
        // TODO: Or is it better to just hardcode the ADC scaling factor?
        self.battery_voltage = adc.calculate_voltage(buffer[0]) * 30;
        self.motor_current = adc.calculate_voltage(buffer[1]);
        self.backup_battery_voltage = adc.calculate_voltage(buffer[2]) * 2;
    }
}

enum AdcDmaState {
    NotStarted(AdcDma<Sequence, Scan>, &'static mut [u16; 3]),
    Started(Transfer<W, &'static mut [u16; 3], AdcDma<Sequence, Scan>>),
    None,
}

impl AdcDmaState {
    fn with(&mut self, f: impl FnOnce(Self) -> Self) {
        let adc_dma = mem::replace(self, AdcDmaState::None);
        let adc_dma = f(adc_dma);
        let _ = mem::replace(self, adc_dma);
    }
}

static SHARED: Mutex<RefCell<Option<Shared>>> = Mutex::new(RefCell::new(None));

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
    green_high: PA10<Alternate<AF2>>,
    blue_high: PA9<Alternate<AF2>>,
    yellow_high: PA8<Alternate<AF2>>,
    green_low: PB15<Alternate<AF2>>,
    blue_low: PB14<Alternate<AF2>>,
    yellow_low: PB13<Alternate<AF2>>,
    emergency_off: PB12<Alternate<AF2>>,
    //pub pwm: Pwm,
}

impl Motor {
    pub fn set_position_power(&mut self, power: i16, position: u8) {
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
        free(|cs| {
            if let Some(shared) = &mut *SHARED.borrow(cs).borrow_mut() {
                let duty_max = shared.pwm.duty_max();
                let power_max = (duty_max / 2) as i32;
                let y = y as i32 * power_max / 1000;
                let b = b as i32 * power_max / 1000;
                let g = g as i32 * power_max / 1000;
                let y = clamp((y + power_max) as u16, 10, duty_max - 10);
                let b = clamp((b + power_max) as u16, 10, duty_max - 10);
                let g = clamp((g + power_max) as u16, 10, duty_max - 10);
                shared.pwm.set_duty_cycles(y, b, g);
            }
        });
    }
}

pub struct Pwm {
    timer: TIMER0,
    clocks: Clocks,
    auto_reload_value: u16,
}

#[interrupt]
fn TIMER0_BRK_UP_TRG_COM() {
    free(|cs| {
        if let Some(shared) = &mut *SHARED.borrow(cs).borrow_mut() {
            let intf = &shared.pwm.timer.intf;
            if intf.read().upif().is_update_pending() {
                shared.adc_dma.with(move |adc_dma| {
                    if let AdcDmaState::NotStarted(mut adc_dma, buffer) = adc_dma {
                        // Enable interrupts
                        adc_dma.channel.listen(Event::TransferComplete);
                        // Trigger ADC
                        AdcDmaState::Started(adc_dma.read(buffer))
                    } else {
                        adc_dma
                    }
                });
                // Clear timer update interrupt flag
                intf.modify(|_, w| w.upif().clear());
            }
        }
    });
}

#[interrupt]
fn DMA_CHANNEL0() {
    free(|cs| {
        if let Some(shared) = &mut *SHARED.borrow(cs).borrow_mut() {
            let last_adc_readings = &mut shared.last_adc_readings;
            shared.adc_dma.with(move |adc_dma| {
                if let AdcDmaState::Started(transfer) = adc_dma {
                    let (buffer, adc_dma) = transfer.wait();
                    last_adc_readings.update_from_buffer(buffer, adc_dma.as_ref());
                    AdcDmaState::NotStarted(adc_dma, buffer)
                } else {
                    adc_dma
                }
            });
            // TODO: Read hall sensors
            // TODO: set_position based on desired speed and hall sensor reading
        }
    });
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
            clocks,
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

pub struct Hoverboard {
    pub serial: Serial<USART1, PA2<Alternate<AF1>>, PA3<Alternate<AF1>>>,
    pub buzzer: PB10<Output<PushPull>>,
    pub power_latch: PB2<Output<PushPull>>,
    pub power_button: PC15<Input<Floating>>,
    /// This will be low when the charger is connected.
    pub charge_state: PF0<Input<PullUp>>,
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
        timer0: TIMER0,
        dma: DMA,
        adc: ADC,
        ahb: &mut AHB,
        apb1: &mut APB1,
        apb2: &mut APB2,
        clocks: Clocks,
    ) -> Hoverboard {
        let mut gpioa = gpioa.split(ahb);
        let mut gpiob = gpiob.split(ahb);
        let mut gpioc = gpioc.split(ahb);
        let mut gpiof = gpiof.split(ahb);

        // USART
        let tx =
            gpioa
                .pa2
                .into_alternate(&mut gpioa.config, PullMode::Floating, OutputMode::PushPull);
        let rx =
            gpioa
                .pa3
                .into_alternate(&mut gpioa.config, PullMode::Floating, OutputMode::PushPull);

        // DMA controller
        let dma = dma.split(ahb);

        // ADC
        let mut adc = Adc::new(adc, apb2, clocks);
        let battery_voltage = gpioa.pa4.into_analog(&mut gpioa.config);
        let motor_current = gpioa.pa6.into_analog(&mut gpioa.config);
        adc.set_sample_time(&battery_voltage, SampleTime::Cycles13_5);
        adc.set_sample_time(&motor_current, SampleTime::Cycles13_5);
        adc.set_sample_time(&VBat, SampleTime::Cycles13_5);
        adc.enable_vbat(true);
        let mut sequence = Sequence::default();
        sequence.add_pin(battery_voltage).ok().unwrap();
        sequence.add_pin(motor_current).ok().unwrap();
        sequence.add_pin(VBat).ok().unwrap();
        let adc = adc.with_regular_sequence(sequence);
        let adc_dma = adc.with_scan_dma(dma.0, CTN_A::SINGLE, None);
        let adc_dma_buffer = singleton!(: [u16; 3] = [0; 3]).unwrap();

        let pwm = Pwm::new(timer0, MOTOR_PWM_FREQ_HERTZ.hz(), clocks, apb2);

        free(move |cs| {
            SHARED.borrow(cs).replace(Some(Shared {
                pwm,
                adc_dma: AdcDmaState::NotStarted(adc_dma, adc_dma_buffer),
                last_adc_readings: AdcReadings::default(),
            }))
        });

        unsafe {
            NVIC::unmask(Interrupt::TIMER0_BRK_UP_TRG_COM);
            NVIC::unmask(Interrupt::DMA_CHANNEL0);
        }

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

    pub fn adc_readings(&self) -> AdcReadings {
        free(|cs| {
            if let Some(shared) = &mut *SHARED.borrow(cs).borrow_mut() {
                shared.last_adc_readings.clone()
            } else {
                AdcReadings::default()
            }
        })
    }
}
