use serialport::{SerialPort, SerialPortSettings};
use st3215::{
    messages::Instruction, messages::InstructionPacket, messages::ReplyPacket,
    messages::ServoIdOrBroadcast, registers::Register,
};
use std::{env, time::Duration};

fn parse_hex(input: &str) -> u8 {
    assert!(
        input.starts_with("0x"),
        "Input must start with '0x'. Received: {}",
        input
    );
    u8::from_str_radix(&input[2..], 16).expect("Input must be a valid hexadecimal number")
}

fn parse_hex_u16(input: &str) -> u16 {
    assert!(
        input.starts_with("0x"),
        "Input must start with '0x'. Received: {}",
        input
    );
    u16::from_str_radix(&input[2..], 16).expect("Input must be a valid hexadecimal number")
}

/**
 * This example is an early PoC that the rest of the library works.
 * It is sync because that's the first serial port lib that I found.
 * In practice, I prefer the interface provided by ServoBusAsync.
 * The core of the library (Register, InstructionPacket, ReplyPacket, ...) is written in a sans-io
 * style for the most part, so it shouldn't be too hard to maintain sync and async frontends.
 * If you want to use this library in a sync context, I would recommend copy-pasting that to use as
 * your base, and making a ServoBusBlocking struct with a similar API.
 * I would happily accept patches that implement a ServoBusBlocking struct, but I don't have a use
 * for it at the moment, so it's quite low on my priority list.
 */
fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 4 {
        // TODO:
        // {this} <serial_port_path> RegWriteData <servo_id>
        // {this} <serial_port_path> Action <servo_id>
        // {this} <serial_port_path> SyncWrite <servo_id>
        eprintln!(
            r#"Usage:
            {this} <serial_port_path> ping <servo_id>
            {this} <serial_port_path> read <servo_id> <head_address>
            {this} <serial_port_path> write <servo_id> <head_address> <data>
            {this} <serial_port_path> reset <servo_id>
            "#,
            this = args[0]
        );
        std::process::exit(1);
    }
    let mut serial_port_path: &str = &args[1];
    let mut baud_rate: u32 = 1_000_000;
    if let Some((path, baud)) = serial_port_path.rsplit_once(':') {
        if let Ok(baud) = baud.parse() {
            baud_rate = baud;
            serial_port_path = path;
        } else {
            eprintln!("Invalid baud rate: {}", baud);
            std::process::exit(1);
        }
    }
    let command = &args[2];
    let id = {
        let input = &args[3];
        if input == "BROADCAST" {
            ServoIdOrBroadcast::BROADCAST
        } else if input.starts_with("0x") {
            assert!(
                input.starts_with("0x"),
                "Input must start with '0x'. Received: {}",
                input
            );
            u8::from_str_radix(&input[2..], 16)
                .map(ServoIdOrBroadcast)
                .expect("servo_id must be a valid hexadecimal number or BROADCAST")
        } else {
            u8::from_str_radix(&input, 10)
                .map(ServoIdOrBroadcast)
                .expect("servo_id must be a valid decimal or hexadecimal number or BROADCAST")
        }
    };

    if command == "readall" {
        let mut serial_port = serialport::open_with_settings(
            serial_port_path,
            &SerialPortSettings {
                baud_rate,
                timeout: Duration::from_secs(1),
                ..Default::default()
            },
        )
        .expect("Failed to open serial port");

        for register in Register::iter() {
            let packet = InstructionPacket {
                id,
                instruction: Instruction::read_register(register),
            };
            send_and_print_response(&mut serial_port, packet)
        }
    }

    // FIXME: write this using a proper argument parsing library and use named flags instead of
    // positional arguments
    let packet = match command.as_str() {
        "ping" => InstructionPacket {
            id,
            instruction: Instruction::Ping,
        },
        "read" => {
            let head_address = parse_hex(&args[4]);
            let register =
                Register::from_memory_address(head_address).expect("Invalid head address");
            InstructionPacket {
                id,
                instruction: Instruction::read_register(register),
            }
        }
        "write" => {
            let head_address = parse_hex(&args[4]);
            let register =
                Register::from_memory_address(head_address).expect("Invalid head address");
            let value = parse_hex_u16(&args[5]);
            InstructionPacket {
                id,
                instruction: Instruction::write_register(register, value),
            }
        }
        "reset" => InstructionPacket {
            id,
            instruction: Instruction::Reset,
        },
        _ => {
            eprintln!("Invalid command: {}", command);
            std::process::exit(1);
        }
    };

    let mut serial_port = serialport::open_with_settings(
        serial_port_path,
        &SerialPortSettings {
            baud_rate,
            timeout: Duration::from_secs(1),
            ..Default::default()
        },
    )
    .expect("Failed to open serial port");
    dbg!(serial_port.settings());

    send_and_print_response(&mut serial_port, packet);
}

fn send_and_print_response(serial_port: &mut Box<dyn SerialPort>, packet: InstructionPacket) {
    serial_port
        .write_all(&packet.to_buf())
        .expect("Failed to write to serial port");
    let mut serial_port = embedded_io_adapters::std::FromStd::new(serial_port);

    let response = ReplyPacket::read(&mut serial_port).expect("Failed to read response");
    // println!("{:?}", response);
    match packet.instruction {
        Instruction::Ping => {}
        Instruction::ReadData { head_address, .. } => {
            let register =
                Register::from_memory_address(head_address).expect("Invalid head address");

            println!(
                "{:?} is {:?}",
                register,
                response.interpret_as_register(register)
            );
        }
        Instruction::WriteData { .. } => {}
        _ => todo!(),
    }
}
