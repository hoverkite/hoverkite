use super::adc::{AdcDmaState, AdcReadings};
use super::motor::Motor;
use core::cell::RefCell;
use cortex_m::{
    interrupt::{free, Mutex},
    peripheral::NVIC,
};
use gd32f1x0_hal::{
    pac::{interrupt, Interrupt},
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
                shared.adc_dma.trigger_adc();
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
            shared
                .adc_dma
                .read_dma_result(&mut shared.last_adc_readings);

            shared.motor.update();
        }
    });
}

pub fn unmask_interrupts(motor: Motor, adc_dma: AdcDmaState) {
    free(move |cs| {
        SHARED.borrow(cs).replace(Some(Shared {
            motor,
            adc_dma,
            last_adc_readings: AdcReadings::default(),
        }))
    });

    unsafe {
        NVIC::unmask(Interrupt::TIMER0_BRK_UP_TRG_COM);
        NVIC::unmask(Interrupt::DMA_CHANNEL0);
    }
}
