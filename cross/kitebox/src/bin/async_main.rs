#![no_std]
#![no_main]
#![doc = include_str!("../../README.md")]

use embassy_executor::Spawner;
use embassy_futures::select::{select, Either};
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
use st3215::{messages::ServoIdOrBroadcast, registers::Register, servo_bus_async::ServoBusAsync};

const READ_BUF_SIZE: usize = 64;
const SERVO_ID: ServoIdOrBroadcast = ServoIdOrBroadcast(3);

// When you are okay with using a nightly compiler it's better to use https://docs.rs/static_cell/2.1.0/static_cell/macro.make_static.html
macro_rules! mk_static {
    ($t:ty,$val:expr) => {{
        static STATIC_CELL: static_cell::StaticCell<$t> = static_cell::StaticCell::new();
        #[deny(unused_attributes)]
        let x = STATIC_CELL.uninit().write(($val));
        x
    }};
}

#[esp_hal_embassy::main]
async fn main(spawner: Spawner) {
    esp_println::println!("Init!");
    let peripherals = esp_hal::init(esp_hal::Config::default());
    esp_alloc::heap_allocator!(size: 65 * 1024);

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

    let bus = ServoBusAsync::from_uart(servo_bus_uart);

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

    #[allow(non_upper_case_globals)]
    static esp_now_channel: Channel<CriticalSectionRawMutex, TtyCommand, 10> = Channel::new();

    spawner
        .spawn(esp_now_receiver(
            manager,
            receiver,
            esp_now_channel.sender(),
        ))
        .ok();
    spawner.spawn(broadcaster(sender)).ok();

    #[allow(non_upper_case_globals)]
    static tty_channel: Channel<CriticalSectionRawMutex, TtyCommand, 10> = Channel::new();
    spawner
        .spawn(tty_receiver(tty_uart, tty_channel.sender()))
        .unwrap();

    #[allow(non_upper_case_globals)]
    static servo_channel: Channel<CriticalSectionRawMutex, TtyCommand, 10> = Channel::new();
    spawner
        .spawn(servo_bus_writer(servo_channel.receiver(), bus))
        .unwrap();

    spawner
        .spawn(main_loop(
            tty_channel.receiver(),
            esp_now_channel.receiver(),
            servo_channel.sender(),
            manager,
            sender,
        ))
        .unwrap();
}

#[embassy_executor::task]
async fn main_loop(
    // FIXME: is there seriously no way that I can write `tty_channel_receiver: impl ...`?
    tty_channel_receiver: Receiver<'static, CriticalSectionRawMutex, TtyCommand, 10>,
    esp_now_channel_receiver: Receiver<'static, CriticalSectionRawMutex, TtyCommand, 10>,
    servo_channel_sender: Sender<'static, CriticalSectionRawMutex, TtyCommand, 10>,
    manager: &'static EspNowManager<'static>,
    sender: &'static Mutex<NoopRawMutex, EspNowSender<'static>>,
) {
    loop {
        let command = select(
            tty_channel_receiver.receive(),
            esp_now_channel_receiver.receive(),
        )
        .await;

        let command = match command {
            // if it came from tty then forward it
            Either::First(command) => {
                println!("Forwarding command to esp-now: {command:?}");
                match manager
                    .fetch_peer(false)
                    .or_else(|_| manager.fetch_peer(true))
                {
                    Ok(peer) => {
                        let mut sender = sender.lock().await;
                        sender
                            .send_async(&peer.peer_address, &[command.as_u8()])
                            .await
                            .unwrap_or_else(|e| println!("failed to send {command:?}: {e:?}"));
                    }
                    Err(e) => println!("no peer ({e:?}) skipping esp-now sending"),
                };
                command
            }
            Either::Second(command) => command,
        };

        // Always attempt to action the command, because this simplifies local dev.
        servo_channel_sender.send(command).await;
    }
}

#[embassy_executor::task]
async fn servo_bus_writer(
    command_receiver: Receiver<'static, CriticalSectionRawMutex, TtyCommand, 10>,
    mut bus: ServoBusAsync<Uart<'static, Async>>,
) {
    // put the servo in the middle of its range (0,4096)
    bus.write_register(SERVO_ID, Register::TargetLocation, 2048)
        .await
        .unwrap_or_else(|e| println!("no servo available? {e}"));

    loop {
        let command = command_receiver.receive().await;

        println!("Sending command to servo bus: {command:?}");
        let result = match command {
            TtyCommand::Ping => bus.ping_servo(SERVO_ID).await.map(|_| None),
            TtyCommand::Up => bus.rotate_servo(SERVO_ID, 100).await.map(Some),
            TtyCommand::Down => bus.rotate_servo(SERVO_ID, -100).await.map(Some),
            TtyCommand::Left => bus.rotate_servo(SERVO_ID, -10).await.map(Some),
            TtyCommand::Right => bus.rotate_servo(SERVO_ID, 10).await.map(Some),
            TtyCommand::Unrecognised(other) => {
                esp_println::println!(
                    "Unknown command (ascii {other}): {}",
                    char::from_u32(other.into()).unwrap_or('?')
                );
                Ok(None)
            }
        };
        match result {
            Ok(None) => esp_println::println!("Servo command `{command:?}` ok"),
            Ok(Some(val)) => {
                esp_println::println!("Servo command `{command:?}` ok. New value: {val}")
            }
            Err(e) => esp_println::println!("Servo {command:?} error: {}", e),
        };
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
    // FIXME: while we have a healthy peer, maybe we can pause broadcasting.
    let mut ticker = Ticker::every(Duration::from_secs(1));
    loop {
        ticker.next().await;

        // println!("Send Broadcast...");
        let mut sender = sender.lock().await;
        sender
            .send_async(&BROADCAST_ADDRESS, b"Hello.")
            .await
            .unwrap_or_else(|e| println!("Send broadcast status: {:?}", e));
    }
}

#[embassy_executor::task]
async fn esp_now_receiver(
    manager: &'static EspNowManager<'static>,
    mut receiver: EspNowReceiver<'static>,
    sender: Sender<'static, CriticalSectionRawMutex, TtyCommand, 10>,
) {
    loop {
        let r = receiver.receive_async().await;
        if r.info.dst_address == BROADCAST_ADDRESS {
            if !manager.peer_exists(&r.info.src_address) {
                // FIXME: add peers in a more sensible way (pairing based on proximity?)
                // and negotiate some kind of authentication so that we can't be hijacked.
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
        } else {
            let data = r.data();
            println!("Received {:?}", data);
            let command = TtyCommand::read_async(data).await.unwrap();
            sender.send(command).await;
        }
    }
}
