#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_time::{Duration, Timer, WithTimeout};
use esp_backtrace as _;
use esp_hal::{
    timer::timg::TimerGroup,
    uart::{Config, RxConfig, Uart, UartRx, UartTx},
    Async,
};
use st3215::messages::{Instruction, InstructionPacket, ReplyPacket, ServoId, ServoIdOrBroadcast};

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
        .with_tx(peripherals.GPIO19)
        .with_rx(peripherals.GPIO18)
        .into_async();

    let bus = ServoBus::from_uart(uart1);

    spawner.spawn(reader(bus)).ok();
}

#[embassy_executor::task]
async fn reader(mut bus: ServoBus) {
    loop {
        match bus.ping_servo(SERVO_ID).await {
            Ok(id) => esp_println::println!("Servo ID: {:?}", id),
            Err(e) => esp_println::println!("Ping error: {}", e),
        }
        Timer::after(Duration::from_millis(2000)).await; // Delay like in Arduino
    }
}

struct ServoBus {
    rx: UartRx<'static, Async>,
    tx: UartTx<'static, Async>,
}

impl ServoBus {
    fn from_uart(uart: Uart<'static, Async>) -> Self {
        let (rx, tx) = uart.split();
        Self { rx, tx }
    }

    async fn ping_servo(&mut self, servo_id: u8) -> Result<ServoId, &'static str> {
        let command = InstructionPacket {
            id: ServoIdOrBroadcast(servo_id),
            instruction: Instruction::Ping,
        };

        command.write(&mut self.tx).await.unwrap();
        self.tx.flush_async().await.unwrap();

        // Note that UartRx is documented as not being cancel safe, so I'm hoping that if a byte goes
        // missing then we'll just drop whatever we've read so far and return an error.
        match ReplyPacket::read_async(&mut self.rx)
            .with_timeout(SERVO_RESPONSE_TIMEOUT)
            .await
        {
            Ok(Ok(reply)) => Ok(reply.id), // Extract servo ID
            Ok(_) => Err("Ping failed"),
            Err(_) => Err("Ping timeout"),
        }
    }
}
