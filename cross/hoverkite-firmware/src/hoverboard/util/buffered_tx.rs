use super::circular_buffer::CircularBuffer;
use core::{cell::RefCell, fmt};
use cortex_m::{
    asm::wfi,
    interrupt::{free, Mutex},
};
use embedded_io::{ErrorType, Write, WriteReady};

const SERIAL_BUFFER_SIZE: usize = 300;

/// Serial writer for which interrupts can be enabled and disabled.
pub trait Listenable {
    /// Enable interrupt for when when a byte may be written.
    fn listen(&mut self);

    /// Disable interrupt for when a byte may be written.
    fn unlisten(&mut self);
}

pub struct BufferedSerialWriter<W: 'static + Write + WriteReady + Listenable> {
    state: &'static Mutex<RefCell<BufferState<W>>>,
}

impl<W: Write + WriteReady + Listenable> BufferedSerialWriter<W> {
    pub fn new(state: &'static Mutex<RefCell<BufferState<W>>>) -> Self {
        Self { state }
    }
}

impl<W: Write + WriteReady + Listenable> ErrorType for BufferedSerialWriter<W> {
    type Error = W::Error;
}

impl<W: Write + WriteReady + Listenable> WriteReady for BufferedSerialWriter<W> {
    fn write_ready(&mut self) -> Result<bool, Self::Error> {
        free(|cs| {
            let state = &mut *self.state.borrow(cs).borrow_mut();
            Ok(!state.buffer.is_full())
        })
    }
}

impl<W: Write + WriteReady + Listenable> Write for BufferedSerialWriter<W> {
    fn write(&mut self, buffer: &[u8]) -> Result<usize, Self::Error> {
        if buffer.is_empty() {
            return Ok(0);
        }

        loop {
            let mut written = 0;
            free(|cs| {
                let state = &mut *self.state.borrow(cs).borrow_mut();

                // Add as many bytes as possible to the buffer.
                written = state.buffer.add_all(buffer);

                // Try writing the first byte, as an interrupt might not happen if nothing has been
                // written.
                state.try_write();
            });

            if written == 0 {
                // Buffer was full, wait for an interrupt which might indicate that the interrupt
                // handler has sent some bytes before trying again.
                wfi();
            } else {
                return Ok(written);
            }
        }
    }

    fn flush(&mut self) -> Result<(), Self::Error> {
        loop {
            if let Some(result) = free(|cs| {
                let state = &mut *self.state.borrow(cs).borrow_mut();
                if state.buffer.is_empty() {
                    if let Some(writer) = &mut state.writer {
                        Some(writer.flush())
                    } else {
                        Some(Ok(()))
                    }
                } else {
                    None
                }
            }) {
                break result;
            }
            wfi();
        }
    }
}

impl<W: Write + WriteReady + Listenable> fmt::Write for BufferedSerialWriter<W> {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write_all(s.as_bytes()).map_err(|_| fmt::Error)
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

impl<W: Write + WriteReady + Listenable> BufferState<W> {
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
                if let Ok(true) = writer.write_ready() {
                    if writer.write_all(&[byte]).is_ok() {
                        // If the byte was written successfully, remove it from the buffer.
                        self.buffer.take();
                    }
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
