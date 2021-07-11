use core::mem;
use gd32f1x0_hal::adc::{Adc, AdcDma, Scan, Sequence};
use gd32f1x0_hal::dma::{Transfer, W};

#[allow(dead_code)]
const CURRENT_OFFSET_DC: u16 = 1073;

#[derive(Debug, Default, Clone)]
pub struct AdcReadings {
    pub battery_voltage: u16,
    pub motor_current: u16,
    pub backup_battery_voltage: u16,
}

impl AdcReadings {
    pub(super) fn update_from_buffer(&mut self, buffer: &[u16; 3], adc: &Adc) {
        // TODO: Or is it better to just hardcode the ADC scaling factor?
        self.battery_voltage = adc.calculate_voltage(buffer[0]) * 30;
        self.motor_current = adc.calculate_voltage(buffer[1]);
        self.backup_battery_voltage = adc.calculate_voltage(buffer[2]) * 2;
    }
}

pub(super) enum AdcDmaState {
    NotStarted(AdcDma<Sequence, Scan>, &'static mut [u16; 3]),
    Started(Transfer<W, &'static mut [u16; 3], AdcDma<Sequence, Scan>>),
    None,
}

impl AdcDmaState {
    pub(super) fn with(&mut self, f: impl FnOnce(Self) -> Self) {
        let adc_dma = mem::replace(self, AdcDmaState::None);
        let adc_dma = f(adc_dma);
        let _ = mem::replace(self, adc_dma);
    }
}
