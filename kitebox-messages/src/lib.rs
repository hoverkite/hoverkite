#![no_std]

pub mod kitebox_messages_capnp {
    include!(concat!(env!("OUT_DIR"), "/kitebox_messages_capnp.rs"));
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

    #[test]
    fn test_segment_allocator_size_is_big_enough() {
        // FIXME: surely there is a way for capnp to assert this statically?
        assert!(
            SEGMENT_ALLOCATOR_SIZE
                > kitebox_messages_capnp::data::Builder::STRUCT_SIZE.data as usize
                    + kitebox_messages_capnp::data::Builder::STRUCT_SIZE.pointers as usize * 2
                    + kitebox_messages_capnp::data::Builder::STRUCT_SIZE.pointers as usize
                        * kitebox_messages_capnp::axis_data::Builder::STRUCT_SIZE.data as usize
        );
    }

    #[test]
    fn test_message() {
        let mut segment = [0; SEGMENT_ALLOCATOR_SIZE];
        let mut message = capnp::message::Builder::new(SingleSegmentAllocator::new(&mut segment));

        {
            let root: kitebox_messages_capnp::message::Builder = message.init_root();
            let mut data = root.init_body().init_data();
            data.set_time(1000);

            let mut acc = data.reborrow().init_acc();
            acc.set_x(1);
            acc.set_y(-1);
            acc.set_z(0);

            let mut gyr = data.reborrow().init_gyr();
            gyr.set_x(10);
            gyr.set_y(-10);
            gyr.set_z(0);
        }

        let segments = message.get_segments_for_output();

        let message = capnp::message::Reader::new(
            capnp::message::SegmentArray::new(&segments),
            Default::default(),
        );
        let root = message
            .get_root::<kitebox_messages_capnp::message::Reader>()
            .unwrap();

        let data = match root.get_body().which().unwrap() {
            kitebox_messages_capnp::message::body::Which::Data(data) => data.unwrap(),
            kitebox_messages_capnp::message::body::Which::Time(_) => panic!("expected data"),
        };

        assert_eq!(data.get_time(), 1000);

        assert_eq!(data.get_acc().unwrap().get_x(), 1);
        assert_eq!(data.get_acc().unwrap().get_y(), -1);
        assert_eq!(data.get_acc().unwrap().get_z(), 0);

        assert_eq!(data.get_gyr().unwrap().get_x(), 10);
        assert_eq!(data.get_gyr().unwrap().get_y(), -10);
        assert_eq!(data.get_gyr().unwrap().get_z(), 0);
    }
}
