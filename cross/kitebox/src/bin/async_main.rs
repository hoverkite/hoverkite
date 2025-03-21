#![no_std]
#![no_main]
#![doc = include_str!("../../README.md")]

use bmi2::{
    bmi2_async::Bmi2,
    interface::I2cInterface,
    types::{AccBwp, AccConf, Burst, Odr, PerfMode, PwrCtrl},
    I2cAddr,
};
use embassy_embedded_hal::shared_bus::asynch::i2c::I2cDevice;
use embassy_executor::Spawner;
use embassy_futures::select::{select, Either};
use embassy_sync::{
    blocking_mutex::raw::{CriticalSectionRawMutex, NoopRawMutex},
    channel::{Channel, Receiver, Sender},
    mutex::Mutex,
};
use embassy_time::{Duration, Ticker};
use embedded_io_async::Write;
use esp_alloc as _;
use esp_backtrace as _;
use esp_backtrace as _;
use esp_hal::{
    i2c::master::I2c,
    rng::Rng,
    time::{Instant, Rate},
    timer::timg::TimerGroup,
    uart::{UartRx, UartTx},
};
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
use kitebox_messages::{ImuData, Report, ReportMessage, Time};
use st3215::{messages::ServoIdOrBroadcast, registers::Register, servo_bus_async::ServoBusAsync};
use static_cell::StaticCell;

const READ_BUF_SIZE: usize = 64;

type I2cBus = Mutex<NoopRawMutex, I2c<'static, Async>>;
type IMU = Bmi2<I2cInterface<I2cDevice<'static, NoopRawMutex, I2c<'static, Async>>>>;

// Copy-pasta from esp-hal examples.
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

    #[allow(non_upper_case_globals)]
    static from_esp_now_channel: Channel<CriticalSectionRawMutex, TtyCommand, 10> = Channel::new();
    #[allow(non_upper_case_globals)]
    static to_esp_now_channel: Channel<CriticalSectionRawMutex, TtyCommand, 10> = Channel::new();

    spawner
        .spawn(esp_now_reader(
            manager,
            receiver,
            from_esp_now_channel.sender(),
        ))
        .ok();
    spawner
        .spawn(esp_now_writer(
            to_esp_now_channel.receiver(),
            manager,
            sender,
        ))
        .ok();

    let tty_uart = Uart::new(
        peripherals.UART0,
        Config::default()
            .with_rx(RxConfig::default().with_fifo_full_threshold(READ_BUF_SIZE as u16)),
    )
    .unwrap()
    .with_tx(peripherals.GPIO1)
    .with_rx(peripherals.GPIO3)
    .into_async();
    let (tty_rx, tty_tx) = tty_uart.split();

    #[allow(non_upper_case_globals)]
    static from_tty_channel: Channel<CriticalSectionRawMutex, TtyCommand, 10> = Channel::new();
    spawner
        .spawn(tty_reader(tty_rx, from_tty_channel.sender()))
        .unwrap();

    #[allow(non_upper_case_globals)]
    static to_tty_channel: Channel<CriticalSectionRawMutex, ReportMessage, 10> = Channel::new();
    spawner
        .spawn(tty_writer(to_tty_channel.receiver(), tty_tx))
        .unwrap();

    #[allow(non_upper_case_globals)]
    static servo_channel: Channel<CriticalSectionRawMutex, TtyCommand, 10> = Channel::new();
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
    spawner
        .spawn(servo_bus_writer(servo_channel.receiver(), bus))
        .unwrap();

    let i2c = I2c::new(
        peripherals.I2C0,
        esp_hal::i2c::master::Config::default().with_frequency(Rate::from_khz(400)),
    )
    .unwrap()
    .with_sda(peripherals.GPIO26)
    .with_scl(peripherals.GPIO32)
    .into_async();
    static I2C_BUS: StaticCell<I2cBus> = StaticCell::new();
    let i2c_bus = I2C_BUS.init(Mutex::new(i2c));

    let imu_i2c_device = I2cDevice::new(i2c_bus);
    // Sending BMI270_CONFIG_FILE takes sufficiently long (8192 bytes at 31 bytes per transaction)
    // that kitebox-controller has typically had enough time to connect to the UART before we do
    // anything interesting. In theory this is all async so we could chuck it down into the
    // imu_reporter() task and do it in parallel with running the main loop, but I think I prefer it
    // this way.
    let mut imu = Bmi2::new_i2c(imu_i2c_device, I2cAddr::Default, Burst::Other(31));
    imu.init(&bmi2::config::BMI270_CONFIG_FILE).await.unwrap();
    imu.set_acc_conf(AccConf {
        odr: Odr::Odr25,
        bwp: AccBwp::Osr2Avg2,
        filter_perf: PerfMode::Perf,
    })
    .await
    .unwrap();
    imu.set_pwr_ctrl(PwrCtrl {
        aux_en: false,
        gyr_en: true,
        acc_en: true,
        temp_en: true,
    })
    .await
    .unwrap();

    spawner
        .spawn(imu_reporter(imu, to_tty_channel.sender()))
        .unwrap();

    spawner
        .spawn(main_loop(
            from_tty_channel.receiver(),
            to_tty_channel.sender(),
            from_esp_now_channel.receiver(),
            to_esp_now_channel.sender(),
            servo_channel.sender(),
        ))
        .unwrap();
}

#[embassy_executor::task]
async fn main_loop(
    from_tty_channel_receiver: Receiver<'static, CriticalSectionRawMutex, TtyCommand, 10>,
    to_tty_channel_sender: Sender<'static, CriticalSectionRawMutex, ReportMessage, 10>,
    from_esp_now_channel_receiver: Receiver<'static, CriticalSectionRawMutex, TtyCommand, 10>,
    to_esp_now_channel_sender: Sender<'static, CriticalSectionRawMutex, TtyCommand, 10>,
    servo_channel_sender: Sender<'static, CriticalSectionRawMutex, TtyCommand, 10>,
) {
    loop {
        let command = select(
            from_tty_channel_receiver.receive(),
            from_esp_now_channel_receiver.receive(),
        )
        .await;

        let command = match command {
            // if it came from tty then forward it
            Either::First(command) => {
                println!("Forwarding command to esp-now: {command:?}");
                to_esp_now_channel_sender.send(command).await;
                command
            }
            Either::Second(command) => command,
        };

        // Always attempt to action the command, because this simplifies local dev.
        servo_channel_sender.send(command).await;

        // Always report the time after every command
        let now = Instant::now().duration_since_epoch().as_micros();
        let message = ReportMessage {
            report: Report::Time(Time { time: now }),
        };
        to_tty_channel_sender.send(message).await;
    }
}

#[embassy_executor::task]
async fn servo_bus_writer(
    command_receiver: Receiver<'static, CriticalSectionRawMutex, TtyCommand, 10>,
    mut bus: ServoBusAsync<Uart<'static, Async>>,
) {
    let mut maybe_servo_id = bus.ping_servo(ServoIdOrBroadcast::BROADCAST).await.ok();

    if let Some(servo_id) = maybe_servo_id {
        // put the servo in the middle of its range (0,4096)
        bus.write_register(servo_id.into(), Register::TargetLocation, 2048)
            .await
            .unwrap_or_else(|e| println!("no servo available? {e}"));
    }

    loop {
        let command = command_receiver.receive().await;
        let servo_id = match maybe_servo_id {
            Some(id) => id,
            None => {
                maybe_servo_id = bus.ping_servo(ServoIdOrBroadcast::BROADCAST).await.ok();
                match maybe_servo_id {
                    Some(id) => id,
                    None => {
                        let len = command_receiver.len();
                        println!("Could not find servo. Dropping {command:?} and {len} others.");
                        command_receiver.clear();
                        continue;
                    }
                }
            }
        };

        println!("Sending command to servo bus: {command:?}");
        let result = match command {
            TtyCommand::Ping => bus.ping_servo(servo_id.into()).await.map(|_| None),
            TtyCommand::Up => bus.rotate_servo(servo_id, 100).await.map(Some),
            TtyCommand::Down => bus.rotate_servo(servo_id, -100).await.map(Some),
            TtyCommand::Left => bus.rotate_servo(servo_id, -10).await.map(Some),
            TtyCommand::Right => bus.rotate_servo(servo_id, 10).await.map(Some),
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
            // FIXME: handle timeout error here and maybe clear maybe_servo_id?
            Err(e) => esp_println::println!("Servo {command:?} error: {}", e),
        };
    }
}

// This just parses commands from the tty uart and shovels them onto a channel.
#[embassy_executor::task]
async fn tty_reader(
    mut tty_rx: UartRx<'static, Async>,
    from_tty_channel_sender: Sender<'static, CriticalSectionRawMutex, TtyCommand, 10>,
) {
    println!("tty_reader");
    loop {
        let command = TtyCommand::read_async(&mut tty_rx)
            .await
            .expect("should be able to read command from tty (usb uart)");
        println!("received from tty: {command:?}");

        from_tty_channel_sender.send(command).await;
    }
}

#[embassy_executor::task]
async fn tty_writer(
    to_tty_channel_receiver: Receiver<'static, CriticalSectionRawMutex, ReportMessage, 10>,
    mut tty_tx: UartTx<'static, Async>,
) {
    let mut slice = [0u8; ReportMessage::SEGMENT_ALLOCATOR_SIZE];
    loop {
        let message = to_tty_channel_receiver.receive().await;

        let bytes_to_send = message.to_slice(&mut slice);

        // a '#' followed by a capnproto message, using the recommended serialization scheme from
        // https://capnproto.org/encoding.html#serialization-over-a-stream
        tty_tx.write_all(b"#").await.unwrap();
        tty_tx.write_all(&0u32.to_le_bytes()).await.unwrap();
        tty_tx
            .write_all(&(bytes_to_send.len() as u32).to_le_bytes())
            .await
            .unwrap();
        tty_tx.write_all(bytes_to_send).await.unwrap();
        tty_tx.write_all(b"\n").await.unwrap();
    }
}

#[embassy_executor::task]
async fn esp_now_writer(
    to_esp_now_channel_receiver: Receiver<'static, CriticalSectionRawMutex, TtyCommand, 10>,
    manager: &'static EspNowManager<'static>,
    mut sender: EspNowSender<'static>,
) {
    let mut broadcast_ticker = Ticker::every(Duration::from_secs(1));
    loop {
        match select(
            broadcast_ticker.next(),
            to_esp_now_channel_receiver.receive(),
        )
        .await
        {
            Either::First(_) => {
                // FIXME: while we have a healthy peer, maybe we can pause broadcasting.
                sender
                    .send_async(&BROADCAST_ADDRESS, b"Hello.")
                    .await
                    .unwrap_or_else(|e| println!("Send broadcast status: {:?}", e));
            }
            Either::Second(command) => {
                match manager
                    .fetch_peer(false)
                    .or_else(|_| manager.fetch_peer(true))
                {
                    Ok(peer) => {
                        sender
                            .send_async(&peer.peer_address, &[command.as_u8()])
                            .await
                            .unwrap_or_else(|e| println!("failed to send {command:?}: {e:?}"));
                    }
                    Err(e) => println!("no peer ({e:?}) skipping esp-now sending"),
                };
            }
        }
    }
}

#[embassy_executor::task]
async fn esp_now_reader(
    manager: &'static EspNowManager<'static>,
    mut receiver: EspNowReceiver<'static>,
    from_esp_now_channel_sender: Sender<'static, CriticalSectionRawMutex, TtyCommand, 10>,
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
            from_esp_now_channel_sender.send(command).await;
        }
    }
}

#[embassy_executor::task]
async fn imu_reporter(
    mut imu: IMU,
    to_tty_channel_sender: Sender<'static, CriticalSectionRawMutex, ReportMessage, 10>,
) {
    println!("imu_reporter");
    let mut ticker = Ticker::every(Duration::from_millis(1000 / 25));

    loop {
        // FIXME: is there a way to await the interrupt from the imu instead,
        // so I don't have to keep the ticker configuration in sync with the `Odr::Odr25` setting.
        ticker.next().await;
        let status = imu.get_status().await.unwrap();
        if status.acc_data_ready {
            let data = imu.get_data().await.unwrap();
            let message = ReportMessage {
                report: Report::ImuData(ImuData {
                    acc: data.acc.into(),
                    gyr: data.gyr.into(),
                    time: data.time,
                }),
            };

            to_tty_channel_sender.send(message).await;
        }
    }
}
