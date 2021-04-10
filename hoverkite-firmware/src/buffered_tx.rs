use core::{cell::RefCell, cmp::min, fmt};
use cortex_m::interrupt::{free, Mutex};
use embedded_hal::serial::Write;

const SERIAL_BUFFER_SIZE: usize = 100;

/// A circular buffer for sending to a serial port.
struct SerialBuffer {
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

pub struct BufferedSerialWriter<W: 'static + Write<u8>> {
    state: &'static Mutex<RefCell<BufferState<W>>>,
}

impl<W: Write<u8>> BufferedSerialWriter<W> {
    pub fn new(state: &'static Mutex<RefCell<BufferState<W>>>) -> Self {
        Self { state }
    }
}

impl<W: Write<u8>> fmt::Write for BufferedSerialWriter<W> {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        let mut bytes = s.as_bytes();
        // Block until all bytes can be added to the buffer. It should be drained by the
        // interrupt handler.
        while bytes.len() > 0 {
            free(|cs| {
                // Add as many bytes as possible to the buffer.
                let state = &mut *self.state.borrow(cs).borrow_mut();
                let written = state.buffer.add(&bytes);
                bytes = &bytes[written..];

                // Try writing the first byte, as an interrupt won't happen if nothing has been
                // written.
                state.try_write();
            });
            // TODO: WFI?
        }
        Ok(())
    }
}

pub struct BufferState<W> {
    buffer: SerialBuffer,
    writer: Option<W>,
}

impl<W> BufferState<W> {
    pub const fn new() -> Self {
        Self {
            buffer: SerialBuffer::new(),
            writer: None,
        }
    }
}

impl<W: Write<u8>> BufferState<W> {
    pub fn set_writer(&mut self, writer: W) {
        self.writer = Some(writer);
    }

    /// If the writer is set and there's data in the buffer waiting to be written, try
    pub fn try_write(&mut self) {
        if let Some(writer) = &mut self.writer {
            // If there's a byte to write, try writing it.
            if let Some(byte) = self.buffer.peek() {
                if writer.write(byte).is_ok() {
                    // If the byte was written successfully, remove it from the buffer.
                    self.buffer.take();
                }
            }
        }
    }
}
