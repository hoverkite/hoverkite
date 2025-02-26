#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_time::{Duration, Timer, WithTimeout};
use embedded_io_async::{Read, Write};
use esp_backtrace as _;
use esp_hal::{
    timer::timg::TimerGroup,
    uart::{Config, RxConfig, Uart, UartRx, UartTx},
    Async,
};

// Constants
const READ_BUF_SIZE: usize = 64;
const SERVO_ID: u8 = 3;
static SERVO_RESPONSE_TIMEOUT: Duration = Duration::from_millis(100);

#[esp_hal_embassy::main]
async fn main(spawner: Spawner) {
    esp_println::println!("Init!");
    let peripherals = esp_hal::init(esp_hal::Config::default());

    let timg0 = TimerGroup::new(peripherals.TIMG0);
    esp_hal_embassy::init(timg0.timer0);

    let config = Config::default()
        .with_baudrate(1_000_000)
        .with_rx(RxConfig::default().with_fifo_full_threshold(READ_BUF_SIZE as u16));

    let uart1 = Uart::new(peripherals.UART1, config)
        .unwrap()
        .with_tx(peripherals.GPIO19) // TX pin
        .with_rx(peripherals.GPIO18) // RX pin
        .into_async();

    let (rx, tx) = uart1.split();

    spawner.spawn(reader(rx, tx)).ok();
}

#[embassy_executor::task]
async fn reader(mut rx: UartRx<'static, Async>, mut tx: UartTx<'static, Async>) {
    loop {
        match ping_servo(&mut tx, &mut rx, SERVO_ID).await {
            Ok(id) => esp_println::println!("Servo ID: {}", id),
            Err(e) => esp_println::println!("Ping error: {}", e),
        }
        Timer::after(Duration::from_millis(2000)).await; // Delay like in Arduino
    }
}

async fn ping_servo(
    tx: &mut UartTx<'static, Async>,
    rx: &mut UartRx<'static, Async>,
    servo_id: u8,
) -> Result<u8, &'static str> {
    // FIXME: stop ignoring servo_id (requires calculating the checksum properly)
    let ping_command = [0xFF, 0xFF, 254, 0x02, 0x01, 254];

    tx.write_all(&ping_command).await.unwrap();
    tx.flush_async().await.unwrap();

    // we're expecting a response like [0xff, 0xff, 0x01, 0x02, 0x00, 0xFC]
    // Note that UartRx is documented as not being cancel safe, so I'm hoping that if a byte goes
    // missing then we'll just drop whatever we've read so far and return an error.
    let mut response_buf = [0u8; 6];
    match rx
        .read_exact(&mut response_buf)
        .with_timeout(SERVO_RESPONSE_TIMEOUT)
        .await
    {
        Ok(Ok(())) => Ok(response_buf[2]), // Extract servo ID
        Ok(_) => Err("Ping failed"),
        Err(_) => Err("Ping timeout"),
    }
}
