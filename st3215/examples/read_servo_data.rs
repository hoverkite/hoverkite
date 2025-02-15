use serialport::SerialPortSettings;
use st3215::{Instruction, InstructionPacket, ServoIdOrBroadcast};
use std::env;

fn parse_hex(input: &str) -> u8 {
    assert!(
        input.starts_with("0x"),
        "Input must start with '0x'. Received: {}",
        input
    );
    u8::from_str_radix(&input[2..], 16).expect("Input must be a valid hexadecimal number")
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 5 {
        eprintln!(
            "Usage: {} <serial_port_path> <servo_id> <head_address> <length>",
            args[0]
        );
        std::process::exit(1);
    }
    let serial_port_path = &args[1];
    let servo_id = parse_hex(&args[2]);
    let head_address = parse_hex(&args[3]);
    let length = parse_hex(&args[4]);

    let packet = InstructionPacket {
        id: ServoIdOrBroadcast(servo_id),
        instruction: Instruction::ReadData {
            head_address,
            length,
        },
    };

    let mut serial_port = serialport::open_with_settings(
        serial_port_path,
        &SerialPortSettings {
            baud_rate: 1_000_000,
            ..Default::default()
        },
    )
    .expect("Failed to open serial port");

    serial_port
        .write_all(&packet.to_buf())
        .expect("Failed to write to serial port");
    let mut serial_port = embedded_io_adapters::std::FromStd::new(serial_port);

    let response = st3215::ReplyPacket::read(&mut serial_port).expect("Failed to read response");

    println!("{:?}", response);
}
