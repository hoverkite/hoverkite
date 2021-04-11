use core::{cell::RefCell, fmt};
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

    /// Try to add the given byte to the buffer. Returns true on success, or false if the buffer
    /// was already full.
    pub fn add(&mut self, byte: u8) -> bool {
        if self.length == self.buffer.len() {
            return false;
        }
        self.buffer[(self.start + self.length) % self.buffer.len()] = byte;
        self.length += 1;
        true
    }

    /// Add as many bytes as possible from the given slice to the buffer. Returns the number of
    /// bytes added.
    pub fn add_all(&mut self, bytes: &[u8]) -> usize {
        let mut added = 0;
        for &byte in bytes {
            if self.add(byte) {
                added += 1;
            } else {
                break;
            }
        }
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

    /// Returns true if there are no bytes waiting to be written.
    pub fn is_empty(&self) -> bool {
        self.length == 0
    }
}

pub trait Listenable {
    fn listen(&mut self);
}

pub struct BufferedSerialWriter<W: 'static + Write<u8>> {
    state: &'static Mutex<RefCell<BufferState<W>>>,
}

impl<W: Write<u8>> BufferedSerialWriter<W> {
    pub fn new(state: &'static Mutex<RefCell<BufferState<W>>>) -> Self {
        Self { state }
    }
}

impl<W: Write<u8> + Listenable> fmt::Write for BufferedSerialWriter<W> {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        let mut bytes = s.as_bytes();
        // Block until all bytes can be added to the buffer. It should be drained by the
        // interrupt handler.
        while bytes.len() > 0 {
            free(|cs| {
                // Add as many bytes as possible to the buffer.
                let state = &mut *self.state.borrow(cs).borrow_mut();
                let written = state.buffer.add_all(&bytes);
                bytes = &bytes[written..];

                // Try writing the first byte, as an interrupt won't happen if nothing has been
                // written.
                state.try_write();

                if !state.is_empty() {
                    if let Some(writer) = &mut state.writer {
                        // Enable interrupts
                        writer.listen();
                        // TODO: Should this be on try_write instead?
                    }
                }
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

    pub fn writer(&mut self) -> Option<&mut W> {
        self.writer.as_mut()
    }

    /// If the writer is set and there's data in the buffer waiting to be written, try writing it.
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

    /// Returns true if there are no bytes waiting to be written.
    pub fn is_empty(&self) -> bool {
        self.buffer.is_empty()
    }
}
