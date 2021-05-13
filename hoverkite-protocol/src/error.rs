use core::str::Utf8Error;

#[derive(displaydoc::Display, Debug, Copy, Clone, Eq, PartialEq)]
pub enum ProtocolError {
    /// message too long
    MessageTooLong,
    /// got an invalid side: `{0}`
    InvalidSide(u8),
    /// got an invalid command: `{0}`
    InvalidCommand(u8),
    /// got an unexpected byte: `{0}`
    InvalidByte(u8),
    /// invalid UTF8: `{0}`
    Utf8Error(Utf8Error),
}

#[cfg(feature = "std")]
impl std::error::Error for ProtocolError {}
