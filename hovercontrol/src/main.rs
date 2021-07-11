mod config;
mod controller;
mod homie;

use crate::config::Config;
use crate::controller::Controller;
use crate::homie::Homie;
use eyre::Report;
use gilrs::Gilrs;
use log::error;
use messages::client::Hoverkite;
use tokio::runtime::Runtime;

const BAUD_RATE: u32 = 115_200;

fn main() -> Result<(), Report> {
    stable_eyre::install()?;
    pretty_env_logger::init();
    color_backtrace::install();

    let config = Config::from_file()?;

    let right_port = serialport::new(&config.right_port, BAUD_RATE)
        .open()
        .map_err(|e| {
            error!(
                "Failed to open right serial port {}: {}",
                config.right_port, e
            )
        })
        .ok();
    let left_port = config.left_port.and_then(|name| {
        serialport::new(&name, BAUD_RATE)
            .open()
            .map_err(|e| error!("Failed to open left serial port {}: {}", name, e))
            .ok()
    });
    let hoverkite = Hoverkite::new(right_port, left_port);

    let gilrs = Gilrs::new().unwrap();

    let runtime = Runtime::new()?;
    let handle = runtime.handle();
    let homie = handle.block_on(Homie::make_homie_device(handle, config.mqtt))?;

    let mut controller = Controller::new(hoverkite, gilrs, homie);
    controller.run()
}
