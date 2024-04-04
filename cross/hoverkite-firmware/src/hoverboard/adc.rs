use core::mem;
use cortex_m::singleton;
use gd32f1x0_hal::{
    adc::{Adc, AdcDma, SampleTime, Scan, Sequence, VBat},
    dma::{self, Event, Transfer, W},
    gpio::{
        gpioa::{PA4, PA6},
        Analog,
    },
    pac::{self, adc::ctl1::Ctn},
    prelude::*,
    rcu::{Clocks, APB2},
};

// TODO: Use this to calculate the motor current.
#[allow(dead_code)]
const CURRENT_OFFSET_DC: u16 = 1073;

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

pub enum AdcDmaState {
    NotStarted(AdcDma<Sequence, Scan>, &'static mut [u16; 3]),
    Started(Transfer<W, &'static mut [u16; 3], AdcDma<Sequence, Scan>>),
    None,
}

impl AdcDmaState {
    pub fn setup(
        adc: pac::Adc,
        battery_voltage: PA4<Analog>,
        motor_current: PA6<Analog>,
        apb2: &mut APB2,
        clocks: Clocks,
        dma_channel: dma::C0,
    ) -> AdcDmaState {
        let mut adc = Adc::new(adc, apb2, clocks);
        adc.set_sample_time(&battery_voltage, SampleTime::Cycles13_5);
        adc.set_sample_time(&motor_current, SampleTime::Cycles13_5);
        adc.set_sample_time(&VBat, SampleTime::Cycles13_5);
        adc.enable_vbat(true);
        let mut sequence = Sequence::default();
        sequence.add_pin(battery_voltage).ok().unwrap();
        sequence.add_pin(motor_current).ok().unwrap();
        sequence.add_pin(VBat).ok().unwrap();
        let adc = adc.with_regular_sequence(sequence);
        let adc_dma = adc.with_scan_dma(dma_channel, Ctn::Single, None);
        let adc_dma_buffer = singleton!(: [u16; 3] = [0; 3]).unwrap();
        AdcDmaState::NotStarted(adc_dma, adc_dma_buffer)
    }

    /// Trigger an ADC read using DMA.
    pub fn trigger_adc(&mut self) {
        self.with(move |adc_dma| {
            if let AdcDmaState::NotStarted(mut adc_dma, buffer) = adc_dma {
                // Enable interrupts
                adc_dma.channel.listen(Event::TransferComplete);
                // Trigger ADC
                AdcDmaState::Started(adc_dma.read(buffer))
            } else {
                adc_dma
            }
        });
    }

    /// Fetch ADC results from the DMA buffer, in response to a DMA interrupt.
    pub fn read_dma_result(&mut self, result: &mut AdcReadings) {
        self.with(move |adc_dma| {
            if let AdcDmaState::Started(transfer) = adc_dma {
                let (buffer, adc_dma) = transfer.wait();
                result.update_from_buffer(buffer, adc_dma.as_ref());
                AdcDmaState::NotStarted(adc_dma, buffer)
            } else {
                adc_dma
            }
        });
    }

    fn with(&mut self, f: impl FnOnce(Self) -> Self) {
        let adc_dma = mem::replace(self, AdcDmaState::None);
        let adc_dma = f(adc_dma);
        let _ = mem::replace(self, adc_dma);
    }
}
