use crate::ProtocolError;

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
