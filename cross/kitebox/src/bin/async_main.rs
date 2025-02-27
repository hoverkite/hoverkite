#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_sync::{
    blocking_mutex::raw::CriticalSectionRawMutex,
    channel::{Channel, Receiver, Sender},
};
use esp_backtrace as _;
use esp_hal::{
    timer::timg::TimerGroup,
    uart::{Config, RxConfig, Uart},
    Async,
};
use esp_println::println;
use st3215::registers::Register;

const READ_BUF_SIZE: usize = 64;
const SERVO_ID: u8 = 3;

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

    let bus = kitebox::servo::ServoBus::from_uart(servo_bus_uart);

    #[allow(non_upper_case_globals)]
    static tty_channel: Channel<CriticalSectionRawMutex, TtyCommand, 10> = Channel::new();

    spawner
        .spawn(tty_receiver(tty_uart, tty_channel.sender()))
        .unwrap();

    spawner
        .spawn(main_loop(tty_channel.receiver(), bus))
        .unwrap();
}

#[embassy_executor::task]
async fn main_loop(
    // FIXME: is there seriously no way that I can write `tty_receiver: impl ...`?
    tty_receiver: Receiver<'static, CriticalSectionRawMutex, TtyCommand, 10>,
    mut bus: kitebox::servo::ServoBus,
) {
    // put the servo in the middle of it's range (0,4096)
    bus.write_register(SERVO_ID, Register::TargetLocation, 2048)
        .await
        .unwrap();

    loop {
        let command = tty_receiver.receive().await;
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

// This just parses commands from the tty uart and shovels them onto a channel.
// The esp-hal examples tend to use select(), but TtyCommand::read_async() is not cancel safe,
// and I have a long-standing hatred of select loops.
// (see https://blog.yoshuawuyts.com/futures-concurrency-3/).
//
// I would prefer to do something like:
//     let tty_commands = futures::stream::repeat(()).map(move |()| TtyCommand::read_async(tty_rx));
//     let merged = commands.select(commands_from_espnow)
//     while let Some(command) = merged.next().await { ... }
// I hear that there is a cpu starvation hazard there though, because neither stream is polled while
// the body of the loop is happening (even if it has yielded to the executor). There is a blog post
// about this somewhere...
#[embassy_executor::task]
async fn tty_receiver(
    tty_uart: Uart<'static, Async>,
    // FIXME: replace this with an impl trait?
    sender: Sender<'static, CriticalSectionRawMutex, TtyCommand, 10>,
) {
    // in practice, we use println!() to respond, so we don't need the tx part yet.
    // For some reason, if I just pass tty_rx into this function (rather than the whole tty_uart)
    // then it stops working (as if it's dropping the uart in main() and cleaning things up?)
    let (mut tty_rx, _tty_tx) = tty_uart.split();

    println!("tty_receiver");
    loop {
        let command = TtyCommand::read_async(&mut tty_rx)
            .await
            .expect("should be able to read command from tty (usb uart)");
        println!("received from tty: {command:?}");

        sender.send(command).await;
    }
}

#[derive(Debug)]
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
