#![no_std]

use capnp::message::SingleSegmentAllocator;
use capnp_conv::{capnp_conv, Readable, Writable};

pub mod kitebox_messages_capnp {
    include!(concat!(env!("OUT_DIR"), "/kitebox_messages_capnp.rs"));
}

#[capnp_conv(kitebox_messages_capnp::axis_data)]
#[derive(Debug, PartialEq, Eq)]
pub struct AxisData {
    x: i16,
    y: i16,
    z: i16,
}

#[capnp_conv(kitebox_messages_capnp::imu_data)]
#[derive(Debug, PartialEq, Eq)]
pub struct ImuData {
    acc: AxisData,
    gyr: AxisData,
    time: u32,
}

#[capnp_conv(kitebox_messages_capnp::time)]
#[derive(Debug, PartialEq, Eq)]
pub struct Time {
    pub time: u64,
}

#[capnp_conv(kitebox_messages_capnp::report_message::report)]
#[derive(Debug, PartialEq, Eq)]
pub enum Report {
    ImuData(ImuData),
    Time(Time),
}

#[capnp_conv(kitebox_messages_capnp::report_message)]
#[derive(Debug, PartialEq, Eq)]
pub struct ReportMessage {
    #[capnp_conv(type = "union")]
    pub report: Report,
}

impl ReportMessage {
    // FIXME: if I know that I don't have any arrays in my structs, is there a way to get capnp
    // to generate this max size directly?
    pub const SEGMENT_ALLOCATOR_SIZE: usize = 64;

    pub fn to_slice<'a>(&self, slice: &'a mut [u8]) -> &'a [u8] {
        let mut message_builder = capnp::message::Builder::new(SingleSegmentAllocator::new(slice));

        self.write(message_builder.init_root());
        let len = message_builder.get_segments_for_output()[0].len();
        // HACK: don't run the drop handler, because it will zero out the slice.
        // TODO: decide whether we want to require the user to pass in a Builder or something?
        // I guess we could re-export capnp::message::Builder and even have a
        // ReportMessage::make_builder() helper to create a builder of the correct size?
        core::mem::forget(message_builder);

        return &slice[..len];
    }
    pub fn from_slice(slice: &[u8]) -> Self {
        // We limit ourselves to being able to decode a single segment, because
        // we know that we were encoded with a SingleSegmentAllocator
        let segments = &[slice];
        let message = capnp::message::Reader::new(
            capnp::message::SegmentArray::new(segments),
            Default::default(),
        );
        let root = message
            .get_root::<kitebox_messages_capnp::report_message::Reader>()
            .unwrap();

        ReportMessage::read(root).unwrap()
    }
}

#[cfg(test)]
mod tests {
    extern crate std;
    use std::prelude::rust_2024::*;

    use super::*;
    use capnp::traits::HasStructSize;

    #[test]
    fn test_segment_allocator_size_is_big_enough() {
        // FIXME: surely there is a way for capnp to assert this statically?
        assert!(
            ReportMessage::SEGMENT_ALLOCATOR_SIZE
                > kitebox_messages_capnp::imu_data::Builder::STRUCT_SIZE.data as usize
                    + kitebox_messages_capnp::imu_data::Builder::STRUCT_SIZE.pointers as usize * 2
                    + kitebox_messages_capnp::imu_data::Builder::STRUCT_SIZE.pointers as usize
                        * kitebox_messages_capnp::axis_data::Builder::STRUCT_SIZE.data as usize
        );
    }

    #[test]
    fn test_message() {
        let mut segment = [0; ReportMessage::SEGMENT_ALLOCATOR_SIZE];

        let data = ReportMessage {
            report: Report::ImuData(ImuData {
                acc: AxisData { x: 1, y: -1, z: 0 },
                gyr: AxisData {
                    x: 10,
                    y: -10,
                    z: 0,
                },
                time: 1000,
            }),
        };

        let segment = data.to_slice(&mut segment);

        let round_tripped = ReportMessage::from_slice(segment);

        assert_eq!(round_tripped, data);
    }
}
