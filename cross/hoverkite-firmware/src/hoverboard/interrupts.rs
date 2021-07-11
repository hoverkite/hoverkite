use super::adc::{AdcDmaState, AdcReadings};
use super::motor::Motor;
use core::cell::RefCell;
use cortex_m::{
    interrupt::{free, Mutex},
    peripheral::NVIC,
};
use gd32f1x0_hal::{
    dma::Event,
    pac::{interrupt, Interrupt},
    prelude::*,
    timer,
};

pub struct Shared {
    pub motor: Motor,
    pub adc_dma: AdcDmaState,
    pub last_adc_readings: AdcReadings,
}

pub static SHARED: Mutex<RefCell<Option<Shared>>> = Mutex::new(RefCell::new(None));

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

pub fn unmask_interrupts() {
    unsafe {
        NVIC::unmask(Interrupt::TIMER0_BRK_UP_TRG_COM);
        NVIC::unmask(Interrupt::DMA_CHANNEL0);
    }
}
