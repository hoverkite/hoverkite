use crate::{
    buffered_tx::{try_write, BufferedSerialWriter, SerialBuffer},
    motor::{HallSensors, Motor, Pwm},
};
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
        gpioa::{PA0, PA12, PA15},
        gpiob::{PB10, PB2, PB3},
        gpioc::PC15,
        gpiof::PF0,
        Floating, Input, Output, OutputMode, PullMode, PullUp, PushPull,
    },
    pac::{
        adc::ctl1::CTN_A, interrupt, Interrupt, ADC, DMA, GPIOA, GPIOB, GPIOC, GPIOF, TIMER0,
        USART0, USART1,
    },
    prelude::*,
    rcu::{Clocks, AHB, APB1, APB2},
    serial::{Config, Rx, Serial, Tx},
};

const USART_BAUD_RATE: u32 = 115200;
const MOTOR_PWM_FREQ_HERTZ: u32 = 16000;
const CURRENT_OFFSET_DC: u16 = 1073;

struct Shared {
    motor: Motor,
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
static SERIAL0_BUFFER: Mutex<RefCell<SerialBuffer>> = Mutex::new(RefCell::new(SerialBuffer::new()));
static SERIAL0_TX: Mutex<RefCell<Option<Tx<USART0>>>> = Mutex::new(RefCell::new(None));
static SERIAL1_BUFFER: Mutex<RefCell<SerialBuffer>> = Mutex::new(RefCell::new(SerialBuffer::new()));
static SERIAL1_TX: Mutex<RefCell<Option<Tx<USART1>>>> = Mutex::new(RefCell::new(None));

#[interrupt]
fn USART0() {
    free(|cs| {
        if let Some(tx) = &mut *SERIAL0_TX.borrow(cs).borrow_mut() {
            let buffer = &mut *SERIAL0_BUFFER.borrow(cs).borrow_mut();
            // If there's a byte to write, try writing it.
            try_write(buffer, tx);
        }
    })
}

#[interrupt]
fn USART1() {
    free(|cs| {
        if let Some(tx) = &mut *SERIAL1_TX.borrow(cs).borrow_mut() {
            let buffer = &mut *SERIAL1_BUFFER.borrow(cs).borrow_mut();
            // If there's a byte to write, try writing it.
            try_write(buffer, tx);
        }
    })
}

pub struct Leds {
    pub side: PA0<Output<PushPull>>,
    pub green: PA15<Output<PushPull>>,
    pub orange: PA12<Output<PushPull>>,
    pub red: PB3<Output<PushPull>>,
}

#[interrupt]
fn TIMER0_BRK_UP_TRG_COM() {
    free(|cs| {
        if let Some(shared) = &mut *SHARED.borrow(cs).borrow_mut() {
            let intf = &shared.motor.pwm.timer.intf;
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
            // Fetch ADC readings from the DMA buffer.
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

            shared.motor.update();
        }
    });
}

pub struct Hoverboard {
    pub serial_remote_rx: Rx<USART0>,
    pub serial_remote_writer: BufferedSerialWriter<Tx<USART0>>,
    pub serial_rx: Rx<USART1>,
    pub serial_writer: BufferedSerialWriter<Tx<USART1>>,
    pub buzzer: PB10<Output<PushPull>>,
    pub power_latch: PB2<Output<PushPull>>,
    pub power_button: PC15<Input<Floating>>,
    /// This will be low when the charger is connected.
    pub charge_state: PF0<Input<PullUp>>,
    pub leds: Leds,
}

impl Hoverboard {
    pub fn new(
        gpioa: GPIOA,
        gpiob: GPIOB,
        gpioc: GPIOC,
        gpiof: GPIOF,
        usart0: USART0,
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

        // USART0
        let tx0 =
            gpiob
                .pb6
                .into_alternate(&mut gpiob.config, PullMode::Floating, OutputMode::PushPull);
        let rx0 =
            gpiob
                .pb7
                .into_alternate(&mut gpiob.config, PullMode::Floating, OutputMode::PushPull);
        let (mut serial_remote_tx, serial_remote_rx) = Serial::usart(
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
        serial_remote_tx.listen();
        free(move |cs| SERIAL0_TX.borrow(cs).replace(Some(serial_remote_tx)));
        let serial_remote_writer = BufferedSerialWriter::new(&SERIAL0_BUFFER, &SERIAL0_TX);

        // USART1
        let tx1 =
            gpioa
                .pa2
                .into_alternate(&mut gpioa.config, PullMode::Floating, OutputMode::PushPull);
        let rx1 =
            gpioa
                .pa3
                .into_alternate(&mut gpioa.config, PullMode::Floating, OutputMode::PushPull);
        let (mut serial_tx, serial_rx) = Serial::usart(
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
        serial_tx.listen();
        free(move |cs| SERIAL1_TX.borrow(cs).replace(Some(serial_tx)));
        let serial_writer = BufferedSerialWriter::new(&SERIAL1_BUFFER, &SERIAL1_TX);

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

        let hall_sensors = HallSensors::new(
            gpiob.pb11.into_floating_input(&mut gpiob.config),
            gpiof.pf1.into_floating_input(&mut gpiof.config),
            gpioc.pc14.into_floating_input(&mut gpioc.config),
        );

        let motor = Motor::new(
            // Output speed defaults to 2MHz
            gpioa
                .pa10
                .into_alternate(&mut gpioa.config, PullMode::Floating, OutputMode::PushPull),
            gpioa
                .pa9
                .into_alternate(&mut gpioa.config, PullMode::Floating, OutputMode::PushPull),
            gpioa
                .pa8
                .into_alternate(&mut gpioa.config, PullMode::Floating, OutputMode::PushPull),
            gpiob
                .pb15
                .into_alternate(&mut gpiob.config, PullMode::Floating, OutputMode::PushPull),
            gpiob
                .pb14
                .into_alternate(&mut gpiob.config, PullMode::Floating, OutputMode::PushPull),
            gpiob
                .pb13
                .into_alternate(&mut gpiob.config, PullMode::Floating, OutputMode::PushPull),
            gpiob
                .pb12
                .into_alternate(&mut gpiob.config, PullMode::Floating, OutputMode::PushPull),
            pwm,
            hall_sensors,
        );

        free(move |cs| {
            SHARED.borrow(cs).replace(Some(Shared {
                motor,
                adc_dma: AdcDmaState::NotStarted(adc_dma, adc_dma_buffer),
                last_adc_readings: AdcReadings::default(),
            }))
        });

        unsafe {
            NVIC::unmask(Interrupt::TIMER0_BRK_UP_TRG_COM);
            NVIC::unmask(Interrupt::DMA_CHANNEL0);
            NVIC::unmask(Interrupt::USART1);
        }

        Hoverboard {
            serial_remote_rx,
            serial_remote_writer,
            serial_rx,
            serial_writer,
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

            shared.motor.position
        })
    }

    /// Set the desired power for the motor.
    pub fn set_motor_power(&mut self, power: i16) {
        free(|cs| {
            // SHARED must have been initialised by the time this is called.
            let shared = &mut *SHARED.borrow(cs).borrow_mut();
            let shared = shared.as_mut().unwrap();

            shared.motor.target_power = power;
        })
    }

    /// Set the motor's current position as 0.
    pub fn recentre_motor(&mut self) {
        free(|cs| {
            // SHARED must have been initialised by the time this is called.
            let shared = &mut *SHARED.borrow(cs).borrow_mut();
            let shared = shared.as_mut().unwrap();

            shared.motor.position = 0;
        })
    }
}
