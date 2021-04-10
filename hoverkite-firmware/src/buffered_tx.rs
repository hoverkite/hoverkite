use core::{cell::RefCell, cmp::min, fmt};
use cortex_m::interrupt::{free, Mutex};
use embedded_hal::serial::Write;

const SERIAL_BUFFER_SIZE: usize = 100;

/// A circular buffer for sending to a serial port.
pub struct SerialBuffer {
    buffer: [u8; SERIAL_BUFFER_SIZE],
    start: usize,
    length: usize,
}

impl SerialBuffer {
    pub const fn new() -> Self {
        Self {
            buffer: [0; SERIAL_BUFFER_SIZE],
            start: 0,
            length: 0,
        }
    }

    /// Add as many bytes as possible from the given slice to the buffer. Returns the number of
    /// bytes added.
    pub fn add(&mut self, mut bytes: &[u8]) -> usize {
        let mut added = 0;
        if self.start + self.length < self.buffer.len() {
            let length_at_end = min(self.buffer.len() - self.length, bytes.len());
            self.buffer[self.length..self.length + length_at_end]
                .copy_from_slice(&bytes[0..length_at_end]);
            bytes = &bytes[added..];
            added += length_at_end;
        }
        if self.start > 0 {
            let length_at_start = min(self.start, bytes.len());
            self.buffer[0..length_at_start].copy_from_slice(&bytes[0..length_at_start]);
            added += length_at_start;
        }
        self.length += added;
        added
    }

    /// Take one byte out of the buffer, if it has any.
    pub fn take(&mut self) -> Option<u8> {
        if self.length == 0 {
            None
        } else {
            let byte = self.buffer[self.start];
            self.start = (self.start + 1) % self.buffer.len();
            self.length -= 1;
            Some(byte)
        }
    }

    /// Get the next byte from the buffer, but don't remove it.
    pub fn peek(&self) -> Option<u8> {
        if self.length == 0 {
            None
        } else {
            Some(self.buffer[self.start])
        }
    }
}

pub struct BufferedSerialWriter<TX: 'static> {
    buffer: &'static Mutex<RefCell<SerialBuffer>>,
    tx: &'static Mutex<RefCell<Option<TX>>>,
}

impl<TX: Write<u8>> BufferedSerialWriter<TX> {
    pub fn new(
        buffer: &'static Mutex<RefCell<SerialBuffer>>,
        tx: &'static Mutex<RefCell<Option<TX>>>,
    ) -> Self {
        Self { buffer, tx }
    }
}

impl<TX: Write<u8>> fmt::Write for BufferedSerialWriter<TX> {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        let mut bytes = s.as_bytes();
        // Block until all bytes can be added to the buffer. It should be drained by the
        // interrupt handler.
        while bytes.len() > 0 {
            free(|cs| {
                // Add as many bytes as possible to the buffer.
                let buffer = &mut *self.buffer.borrow(cs).borrow_mut();
                let written = buffer.add(&bytes);
                bytes = &bytes[written..];

                if let Some(tx) = &mut *self.tx.borrow(cs).borrow_mut() {
                    // Try writing the first byte, as an interrupt won't happen if nothing has been
                    // written.
                    try_write(buffer, tx);
                }
            });
            // TODO: WFI?
        }
        Ok(())
    }
}

pub fn try_write(buffer: &mut SerialBuffer, tx: &mut impl Write<u8>) {
    // If there's a byte to write, try writing it.
    if let Some(byte) = buffer.peek() {
        if tx.write(byte).is_ok() {
            // If the byte was written successfully, remove it from the buffer.
            buffer.take();
        }
    }
}
