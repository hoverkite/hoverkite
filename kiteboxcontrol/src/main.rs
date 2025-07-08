use std::{io::Stdin, process::Stdio, time::Duration};

use embassy_executor::Spawner;
use embassy_futures::select::{Either, select};
use embassy_sync::{
    blocking_mutex::raw::CriticalSectionRawMutex,
    channel::{Channel, Receiver, Sender},
};
use log::error;
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

    loop {
        match select(stdin_rx.receive(), tty_rx.receive()).await {
            Either::First(stdin_msg) => {
                tty_tx.try_send(stdin_msg).unwrap();
            }
            Either::Second(tty_msg) => {
                println!("{tty_msg}")
            }
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

fn spawn_tty_rx_channel(
    mut port: Box<dyn SerialPort>,
) -> Receiver<'static, CriticalSectionRawMutex, String, 10> {
    #[allow(non_upper_case_globals)]
    static tty_rx_channel: Channel<CriticalSectionRawMutex, String, 10> = Channel::new();

    let tx = tty_rx_channel.sender();
    std::thread::spawn(move || {
        loop {
            let mut buf = [0u8];
            let count = port.read(&mut buf).unwrap();
            assert_eq!(count, 1);
            match buf[0] {
                b'#' => todo!("read message as capnproto-encoded message"),
                _ => {
                    let mut line = Vec::from(&buf);
                    loop {
                        let count = port.read(&mut buf).unwrap();
                        assert_eq!(count, 1);
                        match buf[0] {
                            b'\n' => break,
                            o => line.push(o),
                        }
                    }
                    tx.try_send(String::from_utf8_lossy(&line).to_string())
                        .unwrap()
                }
            }
        }
    });

    tty_rx_channel.receiver()
}

fn spawn_tty_tx_channel(
    port: Box<dyn SerialPort>,
) -> Sender<'static, CriticalSectionRawMutex, String, 10> {
    #[allow(non_upper_case_globals)]
    static tty_tx_channel: Channel<CriticalSectionRawMutex, String, 10> = Channel::new();

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
    rx: Receiver<'static, CriticalSectionRawMutex, String, 10>,
) {
    loop {
        let msg = rx.receive().await;
        port.write_all(msg.as_bytes()).unwrap();
    }
}
