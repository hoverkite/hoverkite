#![no_std]
#![no_main]
#![doc = include_str!("../../README.md")]

use core::i16;

use bmi2::{
    bmi2_async::Bmi2,
    interface::I2cInterface,
    types::{AccBwp, AccConf, AccRange, Burst, Odr, PerfMode, PwrCtrl},
    I2cAddr,
};
use embassy_embedded_hal::shared_bus::asynch::i2c::I2cDevice;
use embassy_executor::Spawner;
use embassy_futures::select::{select, select3, Either, Either3};
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
    time::Rate,
    timer::timg::TimerGroup,
    uart::{UartRx, UartTx},
};
use esp_hal::{
    uart::{Config, RxConfig, Uart},
    Async,
};
use esp_println::logger::init_logger;
use esp_wifi::{
    esp_now::{EspNowManager, EspNowReceiver, EspNowSender, PeerInfo, BROADCAST_ADDRESS},
    init, EspWifiController,
};
use kitebox::messages::TtyCommand;
use kitebox_messages::{Command, CommandMessage, ImuData, Report, ReportMessage};
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
    init_logger(log::LevelFilter::Info);
    log::info!("Init!");
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
    log::info!("esp-now version {}", esp_now.version().unwrap());

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

    #[allow(non_upper_case_globals)]
    static from_imu_channel: Channel<CriticalSectionRawMutex, ReportMessage, 10> = Channel::new();
    let imu_i2c_device = I2cDevice::new(i2c_bus);
    let imu = Bmi2::new_i2c(imu_i2c_device, I2cAddr::Default, Burst::Other(31));
    spawner
        .spawn(imu_reporter(imu, from_imu_channel.sender()))
        .unwrap();

    spawner
        .spawn(main_loop(
            from_tty_channel.receiver(),
            from_imu_channel.receiver(),
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
    from_imu_channel_receiver: Receiver<'static, CriticalSectionRawMutex, ReportMessage, 10>,
    to_tty_channel_sender: Sender<'static, CriticalSectionRawMutex, ReportMessage, 10>,
    from_esp_now_channel_receiver: Receiver<'static, CriticalSectionRawMutex, TtyCommand, 10>,
    to_esp_now_channel_sender: Sender<'static, CriticalSectionRawMutex, TtyCommand, 10>,
    servo_channel_sender: Sender<'static, CriticalSectionRawMutex, TtyCommand, 10>,
) {
    loop {
        let command = select3(
            from_tty_channel_receiver.receive(),
            from_esp_now_channel_receiver.receive(),
            from_imu_channel_receiver.receive(),
        )
        .await;

        let command = match command {
            // if it came from tty then forward it
            Either3::First(command) => {
                log::info!("Forwarding command to esp-now: {command:?}");
                to_esp_now_channel_sender.send(command).await;
                command
            }
            Either3::Second(command) => command,
            Either3::Third(report) => {
                to_tty_channel_sender.send(report).await;
                if let Report::ImuData(imu_data) = report.report {
                    let command = TtyCommand::Capnp(Command::SetPosition(
                        (f32::max(imu_data.acc.x + 1.0, 0.0) * 4000.0) as i64 as i16,
                    ));
                    to_esp_now_channel_sender.send(command).await;
                    command
                } else {
                    TtyCommand::Unrecognised(b'X')
                }
            }
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
    let mut maybe_servo_id = bus.ping_servo(ServoIdOrBroadcast::BROADCAST).await.ok();

    if let Some(servo_id) = maybe_servo_id {
        bus.write_register(servo_id.into(), Register::MaximumAngleLimitation, 0)
            .await
            .unwrap();
        // can also set AngularResolution to something bigger than 1 if we want to go even furter.
        // might also need LockMark?
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
                        log::info!("Could not find servo. Dropping {command:?} and {len} others.");
                        command_receiver.clear();
                        continue;
                    }
                }
            }
        };

        log::info!("Sending command to servo bus: {command:?}");
        let result = match command {
            TtyCommand::Newline => Ok(None),
            TtyCommand::Ping => bus
                .ping_servo(servo_id.into())
                .await
                .map(|id| Some(id.into())),
            TtyCommand::Up => bus.rotate_servo(servo_id, 100).await.map(Some),
            TtyCommand::Down => bus.rotate_servo(servo_id, -100).await.map(Some),
            TtyCommand::Left => bus.rotate_servo(servo_id, -10).await.map(Some),
            TtyCommand::Right => bus.rotate_servo(servo_id, 10).await.map(Some),
            TtyCommand::Query => bus.query_servo(servo_id).await.map(Some),
            TtyCommand::Capnp(command) => match command {
                kitebox_messages::Command::SetPosition(position) => bus
                    .write_register(servo_id.into(), Register::TargetLocation, position as u16)
                    .await
                    .map(|()| Some(position as u16)),
                kitebox_messages::Command::NudgePosition(increment) => {
                    bus.rotate_servo(servo_id, increment).await.map(Some)
                }
            },
            TtyCommand::Unrecognised(other) => {
                log::info!(
                    "Unknown command (ascii {other}): {}",
                    char::from_u32(other.into()).unwrap_or('?')
                );
                Ok(None)
            }
        };
        match result {
            Ok(None) => log::info!("Servo command `{command:?}` ok"),
            Ok(Some(val)) => {
                log::info!("Servo command `{command:?}` ok. New value: {val}")
            }
            // FIXME: handle timeout error here and maybe clear maybe_servo_id?
            Err(e) => log::info!("Servo {command:?} error: {}", e),
        };
    }
}

// This just parses commands from the tty uart and shovels them onto a channel.
#[embassy_executor::task]
async fn tty_reader(
    mut tty_rx: UartRx<'static, Async>,
    from_tty_channel_sender: Sender<'static, CriticalSectionRawMutex, TtyCommand, 10>,
) {
    log::info!("tty_reader");
    loop {
        let command = TtyCommand::read_async(&mut tty_rx)
            .await
            .expect("should be able to read command from tty (usb uart)");
        if command == TtyCommand::Newline {
            continue;
        }
        log::info!("received from tty: {command:?}");

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
                    .unwrap_or_else(|e| log::info!("Send broadcast status: {:?}", e));
            }
            Either::Second(command) => {
                match manager
                    .fetch_peer(false)
                    .or_else(|_| manager.fetch_peer(true))
                {
                    Ok(peer) => match command {
                        // FIXME: kill off the non-capnp commands and write a converter in
                        // kiteboxcontrol if convenience is important.
                        TtyCommand::Capnp(command) => {
                            let mut buf = [0u8; CommandMessage::SEGMENT_ALLOCATOR_SIZE];
                            let slice = CommandMessage { command }.to_slice(&mut buf);

                            sender
                                .send_async(&peer.peer_address, slice)
                                .await
                                .unwrap_or_else(|e| {
                                    log::info!("failed to send {command:?}: {e:?}")
                                });
                        }
                        command => {
                            sender
                                .send_async(&peer.peer_address, &[command.as_u8()])
                                .await
                                .unwrap_or_else(|e| {
                                    log::info!("failed to send {command:?}: {e:?}")
                                });
                        }
                    },
                    Err(e) => log::info!("no peer ({e:?}) skipping esp-now sending"),
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
                log::info!("Added peer {:?}", r.info.src_address);
            }
        } else {
            let data = r.data();
            log::info!("Received {:?}", data);
            let command = if data.len() == 1 {
                TtyCommand::read_async(data).await.unwrap()
            } else {
                let message = CommandMessage::from_slice(data).unwrap();
                TtyCommand::Capnp(message.command)
            };
            from_esp_now_channel_sender.send(command).await;
        }
    }
}

#[embassy_executor::task]
async fn imu_reporter(
    mut imu: IMU,
    to_tty_channel_sender: Sender<'static, CriticalSectionRawMutex, ReportMessage, 10>,
) {
    log::info!("imu_reporter");
    let mut ticker = Ticker::every(Duration::from_millis(1000 / 25));

    // Sending BMI270_CONFIG_FILE takes sufficiently long (8192 bytes at 31 bytes per transaction)
    // that kitebox-controller has typically had enough time to connect to the UART before we do
    // anything interesting. Hopefully this doesn't cause problems if run in parallel with
    // the main loop.
    if let Err(e) = imu.init(&bmi2::config::BMI270_CONFIG_FILE).await {
        log::info!("could not talk to imu: {e:?}");
        return;
    };
    imu.set_acc_conf(AccConf {
        odr: Odr::Odr25,
        bwp: AccBwp::Osr2Avg2,
        filter_perf: PerfMode::Perf,
    })
    .await
    .unwrap();
    imu.set_acc_range(AccRange::Range2g).await.unwrap();
    // FIXME: imu.set_gyr_range(?). Not sure what the parameter really means.
    imu.set_pwr_ctrl(PwrCtrl {
        aux_en: false,
        gyr_en: true,
        acc_en: true,
        temp_en: true,
    })
    .await
    .unwrap();

    loop {
        // FIXME: is there a way to await the interrupt from the imu instead,
        // so I don't have to keep the ticker configuration in sync with the `Odr::Odr25` setting.
        ticker.next().await;
        let status = imu.get_status().await.unwrap();
        if status.acc_data_ready {
            let data = imu.get_data().await.unwrap();
            let acc = kitebox_messages::AxisData {
                // fixme: AccRange::Range2g.as_number() to make it easier to keep these things in sync?
                x: data.acc.x as f32 * 2f32 / i16::MAX as f32,
                y: data.acc.y as f32 * 2f32 / i16::MAX as f32,
                z: data.acc.z as f32 * 2f32 / i16::MAX as f32,
            };
            let gyr = kitebox_messages::AxisData {
                // fixme: decide how to scale these
                x: data.gyr.x as f32 / i16::MAX as f32,
                y: data.gyr.y as f32 / i16::MAX as f32,
                z: data.gyr.z as f32 / i16::MAX as f32,
            };
            let message = ReportMessage {
                report: Report::ImuData(ImuData {
                    acc,
                    gyr,
                    time: data.time,
                }),
            };

            to_tty_channel_sender.send(message).await;
        }
    }
}
