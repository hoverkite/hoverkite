mod controller;
mod homie;

use crate::controller::Controller;
use crate::homie::Homie;
use eyre::Report;
use gilrs::Gilrs;
use log::error;
use messages::client::Hoverkite;
use std::env;
use std::process::exit;
use tokio::runtime::Runtime;

const BAUD_RATE: u32 = 115_200;

fn main() -> Result<(), Report> {
    stable_eyre::install()?;
    pretty_env_logger::init();
    color_backtrace::install();

    let mut args = env::args();
    let binary_name = args
        .next()
        .ok_or_else(|| eyre::eyre!("Binary name missing"))?;
    if !(1..=2).contains(&args.len()) {
        eprintln!("Usage:");
        eprintln!("  {} <right serial port> [<left serial port>]", binary_name);
        exit(1);
    }
    let right_port_name = args.next().unwrap();
    let left_port_name = args.next();

    let right_port = serialport::new(&right_port_name, BAUD_RATE)
        .open()
        .map_err(|e| {
            error!(
                "Failed to open right serial port {}: {}",
                right_port_name, e
            )
        })
        .ok();
    let left_port = left_port_name.and_then(|name| {
        serialport::new(&name, BAUD_RATE)
            .open()
            .map_err(|e| error!("Failed to open left serial port {}: {}", name, e))
            .ok()
    });

    let runtime = Runtime::new()?;
    let handle = runtime.handle();

    let gilrs = Gilrs::new().unwrap();
    let hoverkite = Hoverkite::new(right_port, left_port);
    let homie = handle.block_on(Homie::make_homie_device(handle))?;

    let mut controller = Controller::new(hoverkite, gilrs, homie);
    controller.run()
}
