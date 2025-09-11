#![no_std]
use postcard::{from_bytes_cobs, to_vec_cobs};
use serde::{Deserialize, Serialize};

pub const MAX_MESSAGE_SIZE: usize = 32;

#[derive(Clone, Copy, Debug, PartialEq, Deserialize, Serialize)]
pub struct AxisData {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

// Shut the hell up about NaNs.
impl Eq for AxisData {}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize, Serialize)]
pub struct ImuData {
    pub acc: AxisData,
    pub gyr: AxisData,
    pub time: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize, Serialize)]
pub struct Time {
    pub time: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize, Serialize)]
pub enum Report {
    ImuData(ImuData),
    Time(Time),
}

impl Report {
    pub fn to_vec<'a>(&self) -> Result<heapless::Vec<u8, MAX_MESSAGE_SIZE>, postcard::Error> {
        to_vec_cobs(self)
    }
    pub fn from_slice(slice: &mut [u8]) -> Result<Self, postcard::Error> {
        from_bytes_cobs(slice)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize, Serialize)]
pub enum Command {
    SetPosition(i16),
    NudgePosition(i16),
}

// FIXME: DRY
impl Command {
    // FIXME: if I know that I don't have any arrays in my structs, is there a way to get capnp
    // to generate this max size directly?
    pub const SEGMENT_ALLOCATOR_SIZE: usize = 128;

    pub fn to_vec<'a>(&self) -> Result<heapless::Vec<u8, MAX_MESSAGE_SIZE>, postcard::Error> {
        to_vec_cobs(self)
    }
    pub fn from_slice(slice: &mut [u8]) -> Result<Self, postcard::Error> {
        from_bytes_cobs(slice)
    }
}

#[cfg(test)]
mod tests {
    extern crate std;
    use std::prelude::rust_2024::*;

    use super::*;
    #[test]
    fn test_command_roundtrip() {
        let cmd = Command::SetPosition(1000);
        let mut encoded = (cmd).to_vec().unwrap();
        let decoded = Command::from_slice(&mut encoded).unwrap();
        assert_eq!(cmd, decoded);
    }

    #[test]
    fn test_report_roundtrip() {
        let report = Report::ImuData(ImuData {
            acc: AxisData {
                x: 1.0,
                y: -1.0,
                z: 0.0,
            },
            gyr: AxisData {
                x: 10.0,
                y: -10.0,
                z: 0.0,
            },
            time: 1000,
        });
        let mut encoded = report.to_vec().unwrap();

        if encoded[..encoded.len() - 1]
            .iter()
            .find(|b| **b == b'\0')
            .is_some()
        {
            panic!("cobs bytes contains null character")
        }
        let decoded = Report::from_slice(&mut encoded).unwrap();
        assert_eq!(report, decoded);
    }
}
