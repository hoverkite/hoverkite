use super::util::buffered_tx::{BufferState, BufferedSerialWriter, Listenable};
use core::cell::RefCell;
use core::ops::Deref;
use cortex_m::{
    interrupt::{free, Mutex},
    peripheral::NVIC,
};
use gd32f1x0_hal::{
    pac::{interrupt, usart0, Interrupt, Usart0, Usart1},
    serial::Tx,
};

static SERIAL0_BUFFER: Mutex<RefCell<BufferState<Tx<Usart0>>>> =
    Mutex::new(RefCell::new(BufferState::new()));
static SERIAL1_BUFFER: Mutex<RefCell<BufferState<Tx<Usart1>>>> =
    Mutex::new(RefCell::new(BufferState::new()));

pub fn setup_usart0_buffered_writer(
    mut serial_remote_tx: Tx<Usart0>,
) -> BufferedSerialWriter<Tx<Usart0>> {
    serial_remote_tx.listen();
    free(move |cs| {
        SERIAL0_BUFFER
            .borrow(cs)
            .borrow_mut()
            .set_writer(serial_remote_tx)
    });
    unsafe {
        NVIC::unmask(Interrupt::USART0);
    }
    BufferedSerialWriter::new(&SERIAL0_BUFFER)
}

pub fn setup_usart1_buffered_writer(
    mut serial_remote_tx: Tx<Usart1>,
) -> BufferedSerialWriter<Tx<Usart1>> {
    serial_remote_tx.listen();
    free(move |cs| {
        SERIAL1_BUFFER
            .borrow(cs)
            .borrow_mut()
            .set_writer(serial_remote_tx)
    });
    unsafe {
        NVIC::unmask(Interrupt::USART1);
    }
    BufferedSerialWriter::new(&SERIAL1_BUFFER)
}

#[interrupt]
fn USART0() {
    free(|cs| {
        SERIAL0_BUFFER.borrow(cs).borrow_mut().try_write();
    })
}

#[interrupt]
fn USART1() {
    free(|cs| {
        SERIAL1_BUFFER.borrow(cs).borrow_mut().try_write();
    })
}

impl<USART: Deref<Target = usart0::RegisterBlock>> Listenable for Tx<USART> {
    fn listen(&mut self) {
        self.listen()
    }

    fn unlisten(&mut self) {
        self.unlisten()
    }
}
