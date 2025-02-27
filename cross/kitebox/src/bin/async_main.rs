#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_time::{Duration, WithTimeout};
use esp_backtrace as _;
use esp_hal::{
    timer::timg::TimerGroup,
    uart::{Config, RxConfig, Uart, UartRx, UartTx},
    Async,
};
use esp_println::println;
use st3215::{
    messages::{Instruction, InstructionPacket, ReplyPacket, ServoId, ServoIdOrBroadcast},
    registers::Register,
};

const READ_BUF_SIZE: usize = 64;
const SERVO_ID: u8 = 3;
static SERVO_RESPONSE_TIMEOUT: Duration = Duration::from_millis(100);

#[esp_hal_embassy::main]
async fn main(spawner: Spawner) {
    esp_println::println!("Init!");
    let peripherals = esp_hal::init(esp_hal::Config::default());

    let timg0 = TimerGroup::new(peripherals.TIMG0);
    esp_hal_embassy::init(timg0.timer0);

    // This is what's exposed to the terminal when you do `cargo run`.
    let tty_uart = Uart::new(
        peripherals.UART0,
        Config::default()
            .with_rx(RxConfig::default().with_fifo_full_threshold(READ_BUF_SIZE as u16)),
    )
    .unwrap()
    .with_tx(peripherals.GPIO1)
    .with_rx(peripherals.GPIO3)
    .into_async();

    let servo_bus_uart = Uart::new(
        peripherals.UART1,
        Config::default()
            .with_baudrate(1_000_000)
            .with_rx(RxConfig::default().with_fifo_full_threshold(READ_BUF_SIZE as u16)),
    )
    .unwrap()
    .with_tx(peripherals.GPIO19)
    .with_rx(peripherals.GPIO18)
    .into_async();

    let bus = ServoBus::from_uart(servo_bus_uart);

    spawner.spawn(main_loop(tty_uart, bus)).ok();
}

#[embassy_executor::task]
async fn main_loop(tty_uart: Uart<'static, Async>, mut bus: ServoBus) {
    // in practice, we use println!() to respond, so we don't need the tx part yet.
    let (mut tty_rx, _tty_tx) = tty_uart.split();

    // put the servo in the middle of it's range (0,4096)
    bus.write_register(SERVO_ID, Register::TargetLocation, 2048)
        .await
        .unwrap();

    loop {
        let command = TtyCommand::read_async(&mut tty_rx)
            .await
            .expect("should be able to read command from tty (usb uart)");
        match command {
            TtyCommand::Ping => match bus.ping_servo(SERVO_ID).await {
                Ok(id) => esp_println::println!("Servo ID: {:?}", id),
                Err(e) => esp_println::println!("Ping error: {}", e),
            },
            TtyCommand::Up => bus.rotate_servo(SERVO_ID, 100).await.unwrap(),
            TtyCommand::Down => bus.rotate_servo(SERVO_ID, -100).await.unwrap(),
            TtyCommand::Left => bus.rotate_servo(SERVO_ID, -10).await.unwrap(),
            TtyCommand::Right => bus.rotate_servo(SERVO_ID, 10).await.unwrap(),
            TtyCommand::Unrecognised(other) => {
                esp_println::println!(
                    "Unknown command (ascii {other}): {}",
                    char::from_u32(other.into()).unwrap_or('?')
                )
            }
        }
    }
}

enum TtyCommand {
    Ping,
    Up,
    Down,
    Left,
    Right,
    // FIXME: make this into an error instead?
    Unrecognised(u8),
}

impl TtyCommand {
    async fn read_async<R: embedded_io_async::Read>(
        mut stream: R,
    ) -> Result<Self, embedded_io_async::ReadExactError<R::Error>> {
        let mut buffer = [0u8; 1];
        stream.read_exact(&mut buffer).await?;

        Ok(match buffer[0] {
            b'p' => Self::Ping,
            b'^' => Self::Up,
            b'v' => Self::Down,
            b'<' => Self::Left,
            b'>' => Self::Right,
            27 => {
                // Escape codes. Used by arrow keys.
                stream.read_exact(&mut buffer).await?;
                match buffer[0] {
                    b'[' => {
                        stream.read_exact(&mut buffer).await?;
                        match buffer[0] {
                            b'A' => Self::Up,
                            b'B' => Self::Down,
                            b'D' => Self::Left,
                            b'C' => Self::Right,
                            other => Self::Unrecognised(other),
                        }
                    }
                    other => Self::Unrecognised(other),
                }
            }
            other => Self::Unrecognised(other),
        })
    }

    // TODO: proptest that TtyCommand::read_async([cmd.as_u8()]).await == cmd for all cmd?
    fn as_u8(&self) -> u8 {
        match self {
            TtyCommand::Ping => b'p',
            TtyCommand::Up => b'^',
            TtyCommand::Down => b'v',
            TtyCommand::Left => b'<',
            TtyCommand::Right => b'>',
            TtyCommand::Unrecognised(_) => b'?',
        }
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
        let reply = ReplyPacket::read_async(&mut self.rx)
            .with_timeout(SERVO_RESPONSE_TIMEOUT)
            .await
            .map_err(|_| "read timeout")?
            .map_err(|_| "read failed")?;

        Ok(reply.id)
    }

    async fn rotate_servo(&mut self, servo_id: u8, increment: i16) -> Result<(), &'static str> {
        let current = self
            .read_register(servo_id, Register::TargetLocation)
            .await?;

        // you can set any u16 in this register, but if you go outside the range 0,4096, it will
        // get stored as you provide it, but won't cause the servo to rotate out of its circle.
        let next = ((current as i16) + increment) as u16;
        self.write_register(servo_id, Register::TargetLocation, next)
            .await?;

        esp_println::println!("TargetLocation {next}",);

        Ok(())
    }

    async fn read_register(
        &mut self,
        servo_id: u8,
        register: Register,
    ) -> Result<u16, &'static str> {
        let command = InstructionPacket {
            id: ServoIdOrBroadcast(servo_id),
            instruction: Instruction::read_register(register),
        };

        command.write(&mut self.tx).await.unwrap();
        self.tx.flush_async().await.unwrap();

        // Note that UartRx is documented as not being cancel safe, so I'm hoping that if a byte goes
        // missing then we'll just drop whatever we've read so far and return an error.
        let reply = ReplyPacket::read_async(&mut self.rx)
            .with_timeout(SERVO_RESPONSE_TIMEOUT)
            .await
            .map_err(|_| "read timeout")?
            .map_err(|_| "read failed")?;

        let parsed = reply.interpret_as_register(register);

        Ok(parsed)
    }

    async fn write_register(
        &mut self,
        servo_id: u8,
        register: Register,
        value: u16,
    ) -> Result<(), &'static str> {
        let command = InstructionPacket {
            id: ServoIdOrBroadcast(servo_id),
            instruction: Instruction::write_register(register, value),
        };

        command.write(&mut self.tx).await.unwrap();
        self.tx.flush_async().await.unwrap();

        // Note that UartRx is documented as not being cancel safe, so I'm hoping that if a byte goes
        // missing then we'll just drop whatever we've read so far and return an error.
        let reply = ReplyPacket::read_async(&mut self.rx)
            .with_timeout(SERVO_RESPONSE_TIMEOUT)
            .await
            .map_err(|_| "read timeout")?
            .map_err(|_| "read failed")?;

        if !reply.servo_status_errors.is_empty() {
            println!("problem after writing {command:?}: {reply:?}")
        }

        Ok(())
    }
}
