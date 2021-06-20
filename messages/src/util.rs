use std::num::NonZeroU32;

use crate::ProtocolError;
use deku::bitvec::{BitSlice, BitVec, Msb0};
use deku::prelude::*;

pub fn ascii_to_bool(char: u8) -> Result<bool, ProtocolError> {
    match char {
        b'1' => Ok(true),
        b'0' => Ok(false),
        b => Err(ProtocolError::InvalidByte(b)),
    }
}

pub fn bool_to_ascii(on: bool) -> u8 {
    if on {
        b'1'
    } else {
        b'0'
    }
}

pub fn read_bool(rest: &BitSlice<Msb0, u8>) -> Result<(&BitSlice<Msb0, u8>, bool), DekuError> {
    let (rest, value) = u8::read(rest, ())?;
    Ok((
        rest,
        ascii_to_bool(value).map_err(|_| DekuError::Assertion("invalid bool".to_string()))?,
    ))
}

pub fn write_bool(output: &mut BitVec<Msb0, u8>, field_a: bool) -> Result<(), DekuError> {
    let value = bool_to_ascii(field_a);
    value.write(output, ())
}

pub fn read_option_nonzerou32(
    rest: &BitSlice<Msb0, u8>,
) -> Result<(&BitSlice<Msb0, u8>, Option<NonZeroU32>), DekuError> {
    let (rest, value) = u32::read(rest, ())?;
    Ok((rest, NonZeroU32::new(value)))
}
