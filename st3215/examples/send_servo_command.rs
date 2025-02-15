use serialport::SerialPortSettings;
use st3215::{
    messages::Instruction, messages::InstructionPacket, messages::ReplyPacket,
    messages::ServoIdOrBroadcast, registers::Register,
};
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
    let serial_port_path = &args[1];
    let command = &args[2];
    let id = ServoIdOrBroadcast::from_hex_string(&args[3])
        .expect("servo_id must be a valid hexadecimal number or BROADCAST");

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
        "write" => todo!(),
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
            baud_rate: 1_000_000,
            ..Default::default()
        },
    )
    .expect("Failed to open serial port");

    serial_port
        .write_all(&packet.to_buf())
        .expect("Failed to write to serial port");
    let mut serial_port = embedded_io_adapters::std::FromStd::new(serial_port);

    let response = ReplyPacket::read(&mut serial_port).expect("Failed to read response");
    println!("{:?}", response);
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
        _ => todo!(),
    }
}
