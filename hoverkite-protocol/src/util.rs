use crate::ParseError;

pub fn ascii_to_bool(char: u8) -> Result<bool, ParseError> {
    match char {
        b'1' => Ok(true),
        b'0' => Ok(false),
        _ => Err(ParseError),
    }
}

pub fn bool_to_ascii(on: bool) -> u8 {
    if on {
        b'1'
    } else {
        b'0'
    }
}
