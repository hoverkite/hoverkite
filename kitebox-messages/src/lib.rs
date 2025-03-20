#![no_std]

use capnp_conv::capnp_conv;

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
struct Time {
    time: u32,
}

#[capnp_conv(kitebox_messages_capnp::report_message::report)]
#[derive(Debug, PartialEq, Eq)]
enum Report {
    ImuData(ImuData),
    Time(Time),
}

#[capnp_conv(kitebox_messages_capnp::report_message)]
#[derive(Debug, PartialEq, Eq)]
struct ReportMessage {
    #[capnp_conv(type = "union")]
    report: Report,
}

// FIXME: if I know that I don't have any arrays in my structs, is there a way to get capnp
// to generate this max size directly?
pub const SEGMENT_ALLOCATOR_SIZE: usize = 64;

#[cfg(test)]
mod tests {
    extern crate std;
    use std::prelude::rust_2024::*;

    use super::*;
    use capnp::message::SingleSegmentAllocator;
    use capnp::traits::HasStructSize;
    use capnp_conv::{Readable, Writable};

    #[test]
    fn test_segment_allocator_size_is_big_enough() {
        // FIXME: surely there is a way for capnp to assert this statically?
        assert!(
            SEGMENT_ALLOCATOR_SIZE
                > kitebox_messages_capnp::imu_data::Builder::STRUCT_SIZE.data as usize
                    + kitebox_messages_capnp::imu_data::Builder::STRUCT_SIZE.pointers as usize * 2
                    + kitebox_messages_capnp::imu_data::Builder::STRUCT_SIZE.pointers as usize
                        * kitebox_messages_capnp::axis_data::Builder::STRUCT_SIZE.data as usize
        );
    }

    #[test]
    fn test_message() {
        let mut segment = [0; SEGMENT_ALLOCATOR_SIZE];
        let mut message = capnp::message::Builder::new(SingleSegmentAllocator::new(&mut segment));

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

        data.write(message.init_root());

        let segments = message.get_segments_for_output();

        let message = capnp::message::Reader::new(
            capnp::message::SegmentArray::new(&segments),
            Default::default(),
        );
        let root = message
            .get_root::<kitebox_messages_capnp::report_message::Reader>()
            .unwrap();

        let round_tripped = ReportMessage::read(root).unwrap();

        assert_eq!(round_tripped, data);
    }
}
