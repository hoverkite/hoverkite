#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_sync::{blocking_mutex::raw::NoopRawMutex, signal::Signal};
use embedded_io::Write;
use esp_backtrace as _;
use esp_hal::clock::CpuClock;
use esp_hal::{
    uart::{Uart, UartRx, UartTx},
    Async,
};
use log::info;
use static_cell::StaticCell;

use embassy_time::{Duration, Timer, WithTimeout};

extern crate alloc;
const READ_BUF_SIZE: usize = 64;

// FIXME: just pass the whole uart in here?
#[embassy_executor::task]
async fn servo_controller(
    mut rx: UartRx<'static, Async>,
    mut tx: UartTx<'static, Async>,
    signal: &'static Signal<NoopRawMutex, usize>,
) {
    use core::fmt::Write;
    embedded_io_async::Write::write(
        &mut tx,
        b"Hello async serial. Enter something ended with EOT (CTRL-D).\r\n",
    )
    .await
    .unwrap();

    const MAX_BUFFER_SIZE: usize = 10 * READ_BUF_SIZE + 16;

    let mut rbuf: [u8; MAX_BUFFER_SIZE] = [0u8; MAX_BUFFER_SIZE];
    let mut offset = 0;

    info!("servo controller task started");

    embedded_io_async::Write::flush(&mut tx).await.unwrap();

    loop {
        const BROADCAST: u8 = 254; // 0xFE

        // seems reasonable to do a blocking write here: if we only manage to send half a message
        // over the uart then we're probably not going to get the servos doing what we want them to anyway?
        tx.write_all(&[0xff, 0xff, BROADCAST, 0x02, 0x01, 0xfB])
            .unwrap();
        embedded_io_async::Write::flush(&mut tx).await.unwrap();

        let r = embedded_io_async::Read::read(&mut rx, &mut rbuf[offset..])
            .with_timeout(Duration::from_secs(1))
            .await;
        match r {
            Ok(Ok(len)) => {
                offset += len;
                esp_println::println!("Read: {len}, data: {:?}", &rbuf[..offset]);
                offset = 0;
                signal.signal(len);
            }
            Ok(Err(e)) => esp_println::println!("RX Error: {:?}", e),
            Err(e) => esp_println::println!("read timeout: {:?}", e),
        }
    }
}

#[esp_hal_embassy::main]
async fn main(spawner: Spawner) {
    // generator version: 0.2.2

    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);

    esp_alloc::heap_allocator!(72 * 1024);

    esp_println::logger::init_logger_from_env();

    let timer0 = esp_hal::timer::timg::TimerGroup::new(peripherals.TIMG1);
    esp_hal_embassy::init(timer0.timer0);

    info!("Embassy initialized!");

    let timer1 = esp_hal::timer::timg::TimerGroup::new(peripherals.TIMG0);
    let _init = esp_wifi::init(
        timer1.timer0,
        esp_hal::rng::Rng::new(peripherals.RNG),
        peripherals.RADIO_CLK,
    )
    .unwrap();

    let config = esp_hal::uart::Config::default()
        .with_baudrate(1_000_000)
        // 8N1
        .with_data_bits(esp_hal::uart::DataBits::_8)
        .with_parity(esp_hal::uart::Parity::None)
        .with_stop_bits(esp_hal::uart::StopBits::_1);

    let uart1 = Uart::new(peripherals.UART1, config)
        .unwrap()
        .with_rx(peripherals.GPIO18)
        .with_tx(peripherals.GPIO19)
        .into_async();

    let _ = spawner;

    let (rx, tx) = uart1.split();

    static SIGNAL: StaticCell<Signal<NoopRawMutex, usize>> = StaticCell::new();
    let signal = &*SIGNAL.init(Signal::new());

    spawner.spawn(servo_controller(rx, tx, &signal)).ok();

    loop {
        info!("Hello world!");
        Timer::after(Duration::from_secs(100)).await;
    }

    // for inspiration have a look at the examples at https://github.com/esp-rs/esp-hal/tree/v0.23.1/examples/src/bin
}
