use embassy_time::Duration;
use embassy_time::WithTimeout;
use esp_println::println;
use st3215::messages::ServoId;
use st3215::messages::{Instruction, InstructionPacket, ReplyPacket, ServoIdOrBroadcast};
use st3215::registers::Register;

static SERVO_RESPONSE_TIMEOUT: Duration = Duration::from_millis(100);

pub struct ServoBusAsync<U: embedded_io_async::Read + embedded_io_async::Write> {
    uart: U,
}

impl<U: embedded_io_async::Read + embedded_io_async::Write> ServoBusAsync<U> {
    pub fn from_uart(uart: U) -> Self {
        Self { uart }
    }

    pub async fn ping_servo(
        &mut self,
        servo_id: ServoIdOrBroadcast,
    ) -> Result<ServoId, &'static str> {
        let command = InstructionPacket {
            id: servo_id,
            instruction: Instruction::Ping,
        };

        command.write(&mut self.uart).await.unwrap();
        self.uart.flush().await.unwrap();

        // Note that UartRx is documented as not being cancel safe, so I'm hoping that if a byte goes
        // missing then we'll just drop whatever we've read so far and return an error.
        let reply = ReplyPacket::read_async(&mut self.uart)
            .with_timeout(SERVO_RESPONSE_TIMEOUT)
            .await
            .map_err(|_| "read timeout")?
            .map_err(|_| "read failed")?;

        Ok(reply.id)
    }

    pub async fn rotate_servo(
        &mut self,
        servo_id: ServoIdOrBroadcast,
        increment: i16,
    ) -> Result<u16, &'static str> {
        // FIXME: I picked u16 arbitrarily because I thought It would cover all 1 and 2 byte registers.
        // This register is actually documented as being a i16 though (minimum_value: -32766, maximum_value: 32766).
        // I wonder if it's possible to guarantee that we have the return type correct at compile time.
        // Potentially we could:
        // * make a trait IntoRegisterEnum with associated type `rust_type` and `into_register_enum(Self) -> Register`
        // * generate zero sized types that impl this trait (e.g. register_types::TargetLocation)
        // * make a read_register<RegisterType: IntoRegisterEnum>(servo_id: ServoIdOrBroadcast, register: RegisterType) -> RegisterType::rust_type
        // In practice, I should probably fix the addition overflow panic first ;-).
        let current = self
            .read_register(servo_id, Register::TargetLocation)
            .await?;

        // you can set any u16 in this register, but if you go outside the range 0,4096, it will
        // get stored as you provide it, but won't cause the servo to rotate out of its circle.
        let next = ((current as i16) + increment) as u16;
        self.write_register(servo_id, Register::TargetLocation, next)
            .await?;

        Ok(next)
    }

    pub async fn read_register(
        &mut self,
        servo_id: ServoIdOrBroadcast,
        register: Register,
    ) -> Result<u16, &'static str> {
        let command = InstructionPacket {
            id: servo_id,
            instruction: Instruction::read_register(register),
        };

        command.write(&mut self.uart).await.unwrap();
        self.uart.flush().await.unwrap();

        // Note that UartRx is documented as not being cancel safe, so I'm hoping that if a byte goes
        // missing then we'll just drop whatever we've read so far and return an error.
        let reply = ReplyPacket::read_async(&mut self.uart)
            .with_timeout(SERVO_RESPONSE_TIMEOUT)
            .await
            .map_err(|_| "read timeout")?
            .map_err(|_| "read failed")?;

        let parsed = reply.interpret_as_register(register);

        Ok(parsed)
    }

    pub async fn write_register(
        &mut self,
        servo_id: ServoIdOrBroadcast,
        register: Register,
        value: u16,
    ) -> Result<(), &'static str> {
        let command = InstructionPacket {
            id: servo_id,
            instruction: Instruction::write_register(register, value),
        };

        command.write(&mut self.uart).await.unwrap();
        self.uart.flush().await.unwrap();

        // Note that UartRx is documented as not being cancel safe, so I'm hoping that if a byte goes
        // missing then we'll just drop whatever we've read so far and return an error.
        let reply = ReplyPacket::read_async(&mut self.uart)
            .with_timeout(SERVO_RESPONSE_TIMEOUT)
            .await
            .map_err(|_| "read timeout")?
            .map_err(|_| "read failed")?;

        if !reply.servo_status_errors.is_empty() {
            println!("problem after writing {command:?}: {reply:?}")
        }

        Ok(())
    }
}
