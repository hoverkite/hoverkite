use super::adc::{AdcDmaState, AdcReadings};
use super::motor::Motor;
use super::util::buffered_tx::BufferState;
use core::cell::RefCell;
use cortex_m::interrupt::{free, Mutex};
use gd32f1x0_hal::{
    dma::Event,
    pac::{interrupt, USART0, USART1},
    prelude::*,
    serial::Tx,
    timer,
};

pub(super) struct Shared {
    pub(super) motor: Motor,
    pub(super) adc_dma: AdcDmaState,
    pub(super) last_adc_readings: AdcReadings,
}

pub(super) static SHARED: Mutex<RefCell<Option<Shared>>> = Mutex::new(RefCell::new(None));
pub(super) static SERIAL0_BUFFER: Mutex<RefCell<BufferState<Tx<USART0>>>> =
    Mutex::new(RefCell::new(BufferState::new()));
pub(super) static SERIAL1_BUFFER: Mutex<RefCell<BufferState<Tx<USART1>>>> =
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
