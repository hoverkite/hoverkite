#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(feature = "std")]
pub mod client;
mod command;
mod error;
mod response;
mod util;

pub use command::{Command, DirectedCommand, Note, TorqueLimits};
pub use embedded_io::ErrorType;
pub use error::ProtocolError;
pub use response::{Response, SideResponse};

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
