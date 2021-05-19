#![cfg_attr(not(feature = "std"), no_std)]

mod command;
mod error;
mod response;
mod util;

pub use command::{Command, DirectedCommand, Note};
pub use error::ProtocolError;
pub use response::{Response, SideResponse};

/// A compatibility shim that unifies std::io::Write and embedded_hal::blocking::serial::Write
// TODO: propose the following impl to embedded_hal crate:
//
// #[cfg(feature = "std")]
// impl<W: std::io::Write> embedded_hal::blocking::serial::Write<u8> for W {...}
#[cfg(feature = "std")]
pub struct WriteCompat<W: std::io::Write>(pub W);

#[cfg(feature = "std")]
impl<W: std::io::Write> embedded_hal::blocking::serial::Write<u8> for WriteCompat<W> {
    type Error = std::io::Error;

    fn bwrite_all(&mut self, buffer: &[u8]) -> Result<(), Self::Error> {
        self.0.write_all(buffer)
    }

    fn bflush(&mut self) -> Result<(), Self::Error> {
        self.0.flush()
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Side {
    Left,
    Right,
}

impl Side {
    pub fn opposite(self) -> Self {
        match self {
            Self::Left => Self::Right,
            Self::Right => Self::Left,
        }
    }

    pub fn parse(byte: u8) -> Result<Self, ProtocolError> {
        match byte {
            b'L' => Ok(Self::Left),
            b'R' => Ok(Self::Right),
            _ => Err(ProtocolError::InvalidSide(byte)),
        }
    }

    pub fn to_byte(self) -> u8 {
        match self {
            Self::Left => b'L',
            Self::Right => b'R',
        }
    }

    pub fn to_char(self) -> char {
        match self {
            Self::Left => 'L',
            Self::Right => 'R',
        }
    }
}
