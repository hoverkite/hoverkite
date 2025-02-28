#![no_std]
#![no_main]
use embassy_executor::Spawner;
use embassy_sync::{
    blocking_mutex::raw::CriticalSectionRawMutex,
    channel::{Channel, Receiver, Sender},
};
use embassy_sync::{blocking_mutex::raw::NoopRawMutex, mutex::Mutex};
use embassy_time::{Duration, Ticker};
use esp_alloc as _;
use esp_backtrace as _;
use esp_backtrace as _;
use esp_hal::{rng::Rng, timer::timg::TimerGroup};
use esp_hal::{
    uart::{Config, RxConfig, Uart},
    Async,
};
use esp_println::println;
use esp_wifi::{
    esp_now::{EspNowManager, EspNowReceiver, EspNowSender, PeerInfo, BROADCAST_ADDRESS},
    init, EspWifiController,
};
use kitebox::messages::TtyCommand;
use st3215::registers::Register;

const READ_BUF_SIZE: usize = 64;
const SERVO_ID: u8 = 3;

// When you are okay with using a nightly compiler it's better to use https://docs.rs/static_cell/2.1.0/static_cell/macro.make_static.html
macro_rules! mk_static {
    ($t:ty,$val:expr) => {{
        static STATIC_CELL: static_cell::StaticCell<$t> = static_cell::StaticCell::new();
        #[deny(unused_attributes)]
        let x = STATIC_CELL.uninit().write(($val));
        x
    }};
}

/**
 * Kitebox firmware for hoverkite 0.2
 *
 * The intention is that we ship exactly the same code to both the ground station and the box in
 * the sky. The ground kitebox might be connected to a computer over usb, but not connected to a
 * servo bus, but that's okay: the algorithm is still:
 * * if you receive anything from tty_uart:
 *   * attempt to forward it over esp now
 *   * attempt to action it via the servo_uart
 * * if you receive anything from esp now:
 *   * attempt to log it over tty_uart (or in practice esp_println::println!() for now)
 *   * attempt to action it via the servo_uart
 * * if any of your attempts fail because there is nothing connected
 *   * that's fine
 *   * maybe we can log it later, or add metrics?
 *
 *              ground kitebox                       sky kitebox
 *             ┌─────────────────────────┐          ┌─────────────────────────┐
 *             │         esp now ────────┼──────────┼───────► esp now         │
 *             │            ▲            │          │           │             │
 *             │            │            │          │           ▼             │
 *             │     ┌─► main_loop()     │          │        main_loop()─┐    │
 *             │     │                   │          │                    ▼    │
 *          ───►tty_uart       servo_uart┼►x      x─►tty_uart       servo_uart┼────►
 *         usb │                         │          │                         │ servo
 *             └─────────────────────────┘          └─────────────────────────┘  bus
 *
 * This is a very similar approach to hoverkite-firmware, but the hoverkite boards are almost
 * completely identical, and I'm working with a mishmash or esp32 devboards.
 *
 * I suspect that the approach will start to fall apart when we add accelerometer-based inputs and
 * sdcard-based logs.
 */
#[esp_hal_embassy::main]
async fn main(spawner: Spawner) {
    esp_println::println!("Init!");
    let peripherals = esp_hal::init(esp_hal::Config::default());

    let timg1 = TimerGroup::new(peripherals.TIMG1);
    esp_hal_embassy::init(timg1.timer0);

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

    let timg0 = TimerGroup::new(peripherals.TIMG0);

    let esp_wifi_ctrl = &*mk_static!(
        EspWifiController<'static>,
        init(
            timg0.timer0,
            Rng::new(peripherals.RNG),
            peripherals.RADIO_CLK,
        )
        .unwrap()
    );
    let wifi = peripherals.WIFI;
    let esp_now = esp_wifi::esp_now::EspNow::new(&esp_wifi_ctrl, wifi).unwrap();
    println!("esp-now version {}", esp_now.version().unwrap());

    let (manager, sender, receiver) = esp_now.split();
    let manager = mk_static!(EspNowManager<'static>, manager);
    let sender = mk_static!(
        Mutex::<NoopRawMutex, EspNowSender<'static>>,
        Mutex::<NoopRawMutex, _>::new(sender)
    );

    spawner.spawn(listener(manager, receiver)).ok();
    spawner.spawn(broadcaster(sender)).ok();

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

#[embassy_executor::task]
async fn broadcaster(sender: &'static Mutex<NoopRawMutex, EspNowSender<'static>>) {
    let mut ticker = Ticker::every(Duration::from_secs(1));
    loop {
        ticker.next().await;

        println!("Send Broadcast...");
        let mut sender = sender.lock().await;
        let status = sender.send_async(&BROADCAST_ADDRESS, b"Hello.").await;
        println!("Send broadcast status: {:?}", status);
    }
}

#[embassy_executor::task]
async fn listener(manager: &'static EspNowManager<'static>, mut receiver: EspNowReceiver<'static>) {
    loop {
        let r = receiver.receive_async().await;
        println!("Received {:?}", r.data());
        if r.info.dst_address == BROADCAST_ADDRESS {
            if !manager.peer_exists(&r.info.src_address) {
                manager
                    .add_peer(PeerInfo {
                        peer_address: r.info.src_address,
                        lmk: None,
                        channel: None,
                        encrypt: false,
                    })
                    .unwrap();
                println!("Added peer {:?}", r.info.src_address);
            }
        }
    }
}
