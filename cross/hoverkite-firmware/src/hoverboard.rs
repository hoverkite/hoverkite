use crate::{
    buffered_tx::{BufferState, BufferedSerialWriter, Listenable},
    motor::{HallSensors, Motor},
};
use core::{cell::RefCell, mem, ops::Deref};
use cortex_m::{
    interrupt::{free, Mutex},
    peripheral::NVIC,
    singleton,
};
use gd32f1x0_hal::{
    adc::{Adc, AdcDma, SampleTime, Scan, Sequence, VBat},
    dma::{Event, Transfer, W},
    gpio::{
        gpioa::{PA0, PA1, PA12, PA15, PA3},
        gpiob::{PB10, PB2, PB3},
        gpioc::PC15,
        gpiof::PF0,
        Alternate, Floating, Input, Output, OutputMode, PullMode, PullUp, PushPull, AF2,
    },
    pac::{
        adc::ctl1::CTN_A, interrupt, usart0, Interrupt, ADC, DMA, GPIOA, GPIOB, GPIOC, GPIOF,
        TIMER0, TIMER1, USART0, USART1,
    },
    prelude::*,
    pwm::{Channel, Pwm},
    rcu::{Clocks, AHB, APB1, APB2},
    serial::{Config, Rx, Serial, Tx},
    time::Hertz,
    timer::{self, Timer},
};

const USART_BAUD_RATE: u32 = 115200;
const MOTOR_PWM_FREQ_HERTZ: u32 = 16000;
#[allow(dead_code)]
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
static SERIAL0_BUFFER: Mutex<RefCell<BufferState<Tx<USART0>>>> =
    Mutex::new(RefCell::new(BufferState::new()));
static SERIAL1_BUFFER: Mutex<RefCell<BufferState<Tx<USART1>>>> =
    Mutex::new(RefCell::new(BufferState::new()));

#[interrupt]
fn USART0() {
    free(|cs| {
        SERIAL0_BUFFER.borrow(cs).borrow_mut().try_write();
    })
}

#[interrupt]
fn USART1() {
    free(|cs| {
        SERIAL1_BUFFER.borrow(cs).borrow_mut().try_write();
    })
}

impl<USART: Deref<Target = usart0::RegisterBlock>> Listenable for Tx<USART> {
    fn listen(&mut self) {
        self.listen()
    }

    fn unlisten(&mut self) {
        self.unlisten()
    }
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
            let pwm = &mut shared.motor.pwm;
            if pwm.is_pending(timer::Event::Update) {
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
                pwm.clear_interrupt_flag(timer::Event::Update);
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

/// This type is a bit bogus, PB10 is the only one that matters.
type BuzzerPwmPins = (
    Option<PA0<Alternate<AF2>>>,
    Option<PA1<Alternate<AF2>>>,
    Option<PB10<Alternate<AF2>>>,
    Option<PA3<Alternate<AF2>>>,
);

/// The buzzer on the secondary board. This should not be used on the primary board.
pub struct Buzzer {
    pwm: Pwm<TIMER1, BuzzerPwmPins>,
}

impl Buzzer {
    fn new(
        timer1: TIMER1,
        buzzer_pin: PB10<Alternate<AF2>>,
        clocks: Clocks,
        apb1: &mut APB1,
    ) -> Self {
        let pins = (None, None, Some(buzzer_pin), None);
        let pwm = Timer::timer1(timer1, &clocks, apb1).pwm(pins, 1.khz());
        Self { pwm }
    }

    /// Set the frequency of the buzzer, or turn it off.
    pub fn set_frequency(&mut self, frequency: Option<impl Into<Hertz>>) {
        if let Some(frequency) = frequency {
            self.pwm.set_period(frequency.into());
            self.pwm.set_duty(Channel::C2, self.pwm.get_max_duty() / 2);
            self.pwm.enable(Channel::C2);
        } else {
            self.pwm.disable(Channel::C2);
        }
    }
}

pub struct Hoverboard {
    pub serial_remote_rx: Rx<USART0>,
    pub serial_remote_writer: BufferedSerialWriter<Tx<USART0>>,
    pub serial_rx: Rx<USART1>,
    pub serial_writer: BufferedSerialWriter<Tx<USART1>>,
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
        timer0: TIMER0,
        timer1: TIMER1,
        dma: DMA,
        adc: ADC,
        ahb: &mut AHB,
        apb1: &mut APB1,
        apb2: &mut APB2,
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
        free(move |cs| {
            SERIAL0_BUFFER
                .borrow(cs)
                .borrow_mut()
                .set_writer(serial_remote_tx)
        });
        let serial_remote_writer = BufferedSerialWriter::new(&SERIAL0_BUFFER);

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
        free(move |cs| SERIAL1_BUFFER.borrow(cs).borrow_mut().set_writer(serial_tx));
        let serial_writer = BufferedSerialWriter::new(&SERIAL1_BUFFER);

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
            NVIC::unmask(Interrupt::USART0);
            NVIC::unmask(Interrupt::USART1);
        }

        Hoverboard {
            serial_remote_rx,
            serial_remote_writer,
            serial_rx,
            serial_writer,
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
