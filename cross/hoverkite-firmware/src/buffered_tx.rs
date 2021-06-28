use crate::circular_buffer::CircularBuffer;
use core::{cell::RefCell, fmt};
use cortex_m::{
    asm::wfi,
    interrupt::{free, Mutex},
};
use embedded_hal::{blocking, serial::Write};
use nb::block;

const SERIAL_BUFFER_SIZE: usize = 300;

/// Serial writer for which interrupts can be enabled and disabled.
pub trait Listenable {
    /// Enable interrupt for when when a byte may be written.
    fn listen(&mut self);

    /// Disable interrupt for when a byte may be written.
    fn unlisten(&mut self);
}

pub struct BufferedSerialWriter<W: 'static + Write<u8> + Listenable> {
    state: &'static Mutex<RefCell<BufferState<W>>>,
}

impl<W: Write<u8> + Listenable> BufferedSerialWriter<W> {
    pub fn new(state: &'static Mutex<RefCell<BufferState<W>>>) -> Self {
        Self { state }
    }

    /// Write the given bytes to the buffer. This will block if there is not enough space in the
    /// buffer.
    pub fn write_bytes(&mut self, mut bytes: &[u8]) {
        // Block until all bytes can be added to the buffer. It should be drained by the
        // interrupt handler.
        while !bytes.is_empty() {
            free(|cs| {
                // Add as many bytes as possible to the buffer.
                let state = &mut *self.state.borrow(cs).borrow_mut();
                let written = state.buffer.add_all(&bytes);
                bytes = &bytes[written..];

                // Try writing the first byte, as an interrupt might not happen if nothing has been
                // written.
                state.try_write();
            });

            if !bytes.is_empty() {
                // Wait for an interrupt.
                wfi();
            }
        }
    }
}

impl<W: Write<u8> + Listenable> blocking::serial::Write<u8> for BufferedSerialWriter<W> {
    type Error = W::Error;

    fn bwrite_all(&mut self, buffer: &[u8]) -> Result<(), Self::Error> {
        self.write_bytes(buffer);
        Ok(())
    }

    fn bflush(&mut self) -> Result<(), Self::Error> {
        block!(self.flush())
    }
}

impl<W: Write<u8> + Listenable> Write<u8> for BufferedSerialWriter<W> {
    type Error = W::Error;

    fn write(&mut self, word: u8) -> nb::Result<(), Self::Error> {
        free(|cs| {
            let state = &mut *self.state.borrow(cs).borrow_mut();
            if state.buffer.is_full() {
                Err(nb::Error::WouldBlock)
            } else {
                self.write_bytes(&[word]);
                Ok(())
            }
        })
    }

    fn flush(&mut self) -> nb::Result<(), Self::Error> {
        free(|cs| {
            let state = &mut *self.state.borrow(cs).borrow_mut();
            if state.buffer.is_empty() {
                if let Some(writer) = &mut state.writer {
                    writer.flush()
                } else {
                    Ok(())
                }
            } else {
                Err(nb::Error::WouldBlock)
            }
        })
    }
}

impl<W: Write<u8> + Listenable> fmt::Write for BufferedSerialWriter<W> {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write_bytes(s.as_bytes());
        Ok(())
    }
}

pub struct BufferState<W> {
    buffer: CircularBuffer<u8, SERIAL_BUFFER_SIZE>,
    writer: Option<W>,
}

impl<W> BufferState<W> {
    pub const fn new() -> Self {
        Self {
            buffer: CircularBuffer::new(),
            writer: None,
        }
    }
}

impl<W: Write<u8> + Listenable> BufferState<W> {
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

            // If there are bytes left to write, enable interrupts, otherwise disable them.
            if self.buffer.is_empty() {
                writer.unlisten();
            } else {
                writer.listen();
            }
        }
    }
}
