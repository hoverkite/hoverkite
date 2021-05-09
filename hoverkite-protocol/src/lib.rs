#![cfg_attr(not(feature = "std"), no_std)]

mod command;
#[cfg(feature = "std")]
mod response;

use nb::Error::{Other, WouldBlock};

pub use command::{Command, SecondaryCommand, SideCommand};
#[cfg(feature = "std")]
pub use response::{Response, SideResponse, UnexpectedResponse};

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

    pub fn parse(byte: u8) -> Result<Self, ParseError> {
        match byte {
            b'L' => Ok(Self::Left),
            b'R' => Ok(Self::Right),
            _ => Err(ParseError),
        }
    }

    pub fn to_byte(self) -> u8 {
        match self {
            Self::Left => b'L',
            Self::Right => b'R',
        }
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct ParseError;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Message {
    Command(Command),
    SecondaryCommand(SecondaryCommand),
    // TODO: add variants for the other things, or create a second enum that contains them.
    // Something like:
    //     LogMessage(LogMessage),
    //     SecondaryLogMessage(SecondaryLogMessage),
    //     CurrentPosition(CurrentPosition),
    //     SecondaryCurrentPosition(SecondaryCurrentPosition),
}

#[cfg(feature = "std")]
impl Message {
    pub fn write_to_std(&self, writer: impl std::io::Write) -> std::io::Result<()> {
        match self {
            Self::Command(c) => c.write_to_std(writer),
            Self::SecondaryCommand(sc) => sc.write_to_std(writer),
        }
    }
}

impl From<SecondaryCommand> for Message {
    fn from(val: SecondaryCommand) -> Self {
        Message::SecondaryCommand(val)
    }
}

impl From<Command> for Message {
    fn from(val: Command) -> Self {
        Message::Command(val)
    }
}

impl Message {
    pub fn parse(buf: &[u8]) -> nb::Result<Self, ParseError> {
        if let [b'F', ref rest @ ..] = *buf {
            if let [forward_length, ref rest @ ..] = *rest {
                let forward_length = forward_length as usize;
                if rest.len() < forward_length {
                    Err(WouldBlock)
                } else if rest.len() == forward_length {
                    match Command::parse(rest) {
                        Ok(command) => Ok(SecondaryCommand(command).into()),
                        // This will happen if the forward_length byte is corrupt.
                        Err(WouldBlock) => Err(Other(ParseError)),
                        Err(e) => Err(e),
                    }
                } else {
                    Err(Other(ParseError))
                }
            } else {
                Err(WouldBlock)
            }
        } else {
            Ok(Command::parse(buf)?.into())
        }
    }
}
