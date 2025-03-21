use std::{io::Stdin, iter::repeat, process::Stdio, time::Duration};

use embassy_executor::Spawner;
use embassy_futures::select::{Either, select};
use embassy_sync::{
    blocking_mutex::raw::CriticalSectionRawMutex,
    channel::{Channel, Receiver, Sender},
};
use kitebox_messages::Report;
use log::error;
use rerun::Vec3D;
use serialport::SerialPort;

const BAUD_RATE: u32 = 115_200;

/**
 * This program is used as the target.xtensa-esp32-none-elf.runner for kitebox.
 *
 * This lets us send binary messages over the usb serial adapter and decode them for displaying
 * to the user or sending to rerun.io as we see fit.
 */
#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let mut args = std::env::args().collect::<Vec<String>>();

    println!("boxctl starting with args {}", args.join(" "));

    if args.get(1) == Some(&"flash".to_string()) {
        // hack because we're forced to cd out of cross/kitebox to run cargo
        let firmware_path = args[2].clone();
        if !std::fs::exists(&firmware_path).unwrap() {
            let new_path = String::from("../cross/kitebox/") + &firmware_path;
            if std::fs::exists(&new_path).unwrap() {
                args[2] = new_path
            } else {
                panic!("can't find {firmware_path} or {new_path}");
            }
        }
        // Forward the "flash" argument + all others on to espflash
        let status = std::process::Command::new("espflash")
            .args(&args[1..])
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .status()
            .expect("Failed to execute espflash flash command");

        assert_eq!(status.code().unwrap(), 0);
    }
    let status = std::process::Command::new("espflash")
        .arg("reset")
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
        .expect("Failed to execute espflash reset command");

    assert_eq!(status.code().unwrap(), 0);

    // FIXME: consider using lsof to find anything holding this file open and killing it before we attempt to flash?
    let port_path = std::env::var("ESPFLASH_PORT").expect("ESPFLASH_PORT env var must be set");
    let port = match serialport::new(&port_path, BAUD_RATE)
        .timeout(Duration::from_secs(60 * 60))
        .open()
    {
        Ok(port) => port,
        Err(e) => {
            error!("Failed to open serial port {}: {}", port_path, e);
            std::process::exit(1);
        }
    };
    let port_tx = port.try_clone().unwrap();

    let stdin_rx = spawn_stdin_channel(std::io::stdin());
    let tty_rx = spawn_tty_rx_channel(port);
    let tty_tx = spawn_tty_tx_channel(port_tx);

    tty_tx.send("p".into()).await;

    let rec = rerun::RecordingStreamBuilder::new("kiteboxcontrol")
        .connect_tcp()
        .unwrap();

    loop {
        match select(stdin_rx.receive(), tty_rx.receive()).await {
            Either::First(stdin_msg) => {
                tty_tx.try_send(stdin_msg.into()).unwrap();
            }
            Either::Second(line_or_report) => match line_or_report {
                LineOrReport::Line(line) => {
                    if line.len() > 0 {
                        println!("{line}")
                    }
                }
                LineOrReport::Report(report) => match report {
                    Report::Time(time) => {
                        println!("time since boot: {:?}", Duration::from_micros(time.time))
                    }
                    Report::ImuData(imu_data) => {
                        rec.log(
                            "imu/acc",
                            &rerun::Arrows3D::from_vectors([Vec3D::new(
                                imu_data.acc.x.into(),
                                imu_data.acc.y.into(),
                                imu_data.acc.z.into(),
                            )]),
                        )
                        .unwrap();

                        rec.log(
                            "imu/gyr",
                            &rerun::Arrows3D::from_vectors([Vec3D::new(
                                imu_data.gyr.x.into(),
                                imu_data.gyr.y.into(),
                                imu_data.gyr.z.into(),
                            )]),
                        )
                        .unwrap();
                    }
                },
            },
        };
    }
}

fn spawn_stdin_channel(stdin: Stdin) -> Receiver<'static, CriticalSectionRawMutex, String, 10> {
    #[allow(non_upper_case_globals)]
    static stdin_channel: Channel<CriticalSectionRawMutex, String, 10> = Channel::new();

    let tx = stdin_channel.sender();
    std::thread::spawn(move || {
        loop {
            let mut buffer = String::new();
            // FIXME(probably never): make stdin unbuffered and read a char at a time,
            // so we can handle arrow keys without needing to press enter, like we used to.
            // In practice, it would be better to just get the accelerometer or game controller
            // working, and stop worrying about the keyboard.
            stdin.read_line(&mut buffer).unwrap();
            tx.try_send(buffer).unwrap();
        }
    });

    stdin_channel.receiver()
}

#[derive(Debug)]
enum LineOrReport {
    Line(String),
    Report(Report),
}

fn spawn_tty_rx_channel(
    mut port: Box<dyn SerialPort>,
) -> Receiver<'static, CriticalSectionRawMutex, LineOrReport, 10> {
    #[allow(non_upper_case_globals)]
    static tty_rx_channel: Channel<CriticalSectionRawMutex, LineOrReport, 10> = Channel::new();

    let tx = tty_rx_channel.sender();
    std::thread::spawn(move || {
        loop {
            let message = read_message_from_port(&mut port);

            tx.try_send(message)
                .unwrap_or_else(|e| println!("message channel full? {e:?}"))
        }
    });

    tty_rx_channel.receiver()
}

fn read_message_from_port(port: &mut Box<dyn SerialPort>) -> LineOrReport {
    let mut buf = [0u8];
    port.read_exact(&mut buf).unwrap();

    match buf[0] {
        b'#' => {
            // The following bytes are a capnproto message, using the recommended
            // serialization scheme from
            // https://capnproto.org/encoding.html#serialization-over-a-stream
            let mut buf = [0u8; 4];
            // N segments - 1 should always be 0 for a SingleSegmentAllocator
            port.read_exact(&mut buf).unwrap();
            if u32::from_le_bytes(buf) != 0 {
                // FIXME: What the hell is going on here? Happens every time I send a SetPosition.
                let mut line = Vec::from(b"garbled report #");
                line.extend_from_slice(&buf);
                // FIXME: BufRead::read_until()?
                loop {
                    port.read_exact(&mut buf).unwrap();
                    match buf[0] {
                        b'\n' => break,
                        o => line.push(o),
                    }
                }
                println!("garbled report: {:?}", &line[b"garbled report #".len()..]);
                return LineOrReport::Line(String::from_utf8_lossy(&line).to_string());
            }

            // FIXME: fuzz this. It might be possible to drop into the middle of a
            // message and interpret it as a message with a huge length, then wait
            // forever for the esp32 to actually send us that much data.
            port.read_exact(&mut buf).unwrap();
            let len = u32::from_le_bytes(buf) as usize;
            let mut buf = repeat(0u8).take(len).collect::<Vec<_>>();
            port.read_exact(&mut buf).unwrap();

            match kitebox_messages::ReportMessage::from_slice(&buf) {
                Ok(message) => LineOrReport::Report(message.report),
                Err(e) => {
                    println!("error decoding message: {e:?}");
                    // skip until the next newline or #. I kind-of wish we were using cobs
                    // or something for our payloading so that recovering was easier.
                    // FIXME: BufRead::skip_until() or something?
                    loop {
                        let mut buf = [0u8];
                        port.read_exact(&mut buf).unwrap();
                        if let b'\n' | b'#' = buf[0] {
                            break;
                        }
                    }
                    LineOrReport::Line("error decoding message".to_string())
                }
            }
        }
        b'\n' => LineOrReport::Line("".to_string()),
        _ => {
            let mut line = Vec::from(&buf);
            // FIXME: BufRead::read_until()?
            loop {
                port.read_exact(&mut buf).unwrap();
                match buf[0] {
                    b'\n' => break,
                    o => line.push(o),
                }
            }
            LineOrReport::Line(String::from_utf8_lossy(&line).to_string())
        }
    }
}

fn spawn_tty_tx_channel(
    port: Box<dyn SerialPort>,
) -> Sender<'static, CriticalSectionRawMutex, Vec<u8>, 10> {
    #[allow(non_upper_case_globals)]
    static tty_tx_channel: Channel<CriticalSectionRawMutex, Vec<u8>, 10> = Channel::new();

    let tx = tty_tx_channel.sender();
    let rx = tty_tx_channel.receiver();
    std::thread::spawn(move || {
        static TTY_TX_THREAD_EXECUTOR: static_cell::StaticCell<embassy_executor::Executor> =
            static_cell::StaticCell::new();
        let executor = TTY_TX_THREAD_EXECUTOR.init(embassy_executor::Executor::new());
        executor.run(|spawner| {
            spawner.spawn(_forward_tty_tx_channel(port, rx)).unwrap();
        })
    });

    tx
}

#[embassy_executor::task]
async fn _forward_tty_tx_channel(
    mut port: Box<dyn SerialPort>,
    rx: Receiver<'static, CriticalSectionRawMutex, Vec<u8>, 10>,
) {
    loop {
        let msg = rx.receive().await;
        port.write_all(&msg).unwrap();
    }
}
