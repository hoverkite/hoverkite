use core::fmt::Display;

use crate::messages::{
    Instruction, InstructionPacket, ReplyPacket, ServoId, ServoIdOrBroadcast, ServoStatusErrors,
};
use crate::registers::Register;
use embassy_time::{Duration, TimeoutError, WithTimeout};

static SERVO_RESPONSE_TIMEOUT: Duration = Duration::from_millis(100);

#[derive(Debug)]
pub enum ServoBusError {
    /// There is a problem with the servo. The operation **may** have succeed, but don't bet on it.
    ServoStatus(ServoStatusErrors),
    /// Timeout
    Timeout,
    Other(&'static str),
}

impl From<TimeoutError> for ServoBusError {
    fn from(_value: TimeoutError) -> Self {
        Self::Timeout
    }
}

impl From<&'static str> for ServoBusError {
    fn from(value: &'static str) -> Self {
        Self::Other(value)
    }
}

impl Display for ServoBusError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{self:?}")
    }
}

impl core::error::Error for ServoBusError {}

/**
 * This is a slightly higher level wrapper around the core of the bus.
 *
 * It is still very much a work in progress. Going forward, it might be worth merging this into
 * an existing dynamixel servo bus crate (e.g. https://github.com/pollen-robotics/rustypot).
 *
 * The core of the library (Register, InstructionPacket, ReplyPacket, ...) is written in a sans-io
 * style for the most part, so it shouldn't be too hard to maintain sync and async frontends.
 * If you want to use this library in a sync context, I would recommend copy-pasting that to use as
 * your base, and making a ServoBusBlocking struct with a similar API.
 * I would happily accept patches that implement a ServoBusBlocking struct, but I don't have a use
 * for it at the moment, so it's quite low on my priority list.
 */
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
    ) -> Result<ServoId, ServoBusError> {
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
            .await?
            // FIXME: this is a `ReadExactError<<U as ErrorType>::Error>``.
            // Find a way to propagate that doesn't tie the api in knots
            .map_err(|_| "read failed")?;

        Ok(reply.id)
    }

    pub async fn rotate_servo(
        &mut self,
        servo_id: ServoId,
        increment: i16,
    ) -> Result<u16, ServoBusError> {
        // FIXME: I picked u16 arbitrarily because I thought It would cover all 1 and 2 byte registers.
        // This register is actually documented as being a i16 though (minimum_value: -32766, maximum_value: 32766).
        // I wonder if it's possible to guarantee that we have the return type correct at compile time.
        // Potentially we could:
        // * make a trait IntoRegisterEnum with associated type `rust_type` and `into_register_enum(Self) -> Register`
        // * generate zero sized types that impl this trait (e.g. register_types::TargetLocation)
        // * make a read_register<RegisterType: IntoRegisterEnum>(servo_id: ServoId, register: RegisterType) -> RegisterType::rust_type
        // In practice, I should probably fix the addition overflow panic first ;-).
        let current = self
            .read_register(servo_id.into(), Register::TargetLocation)
            .await?;

        // you can set any u16 in this register, but if you go outside the range 0,4096, it will
        // get stored as you provide it, but won't cause the servo to rotate out of its circle.
        let next = ((current as i16) + increment) as u16;
        self.write_register(servo_id.into(), Register::TargetLocation, next)
            .await?;

        Ok(next)
    }

    pub async fn read_register(
        &mut self,
        servo_id: ServoIdOrBroadcast,
        register: Register,
    ) -> Result<u16, ServoBusError> {
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
            .await?
            .map_err(|_| "read failed")?;

        if !reply.servo_status_errors.is_empty() {
            return Err(ServoBusError::ServoStatus(reply.servo_status_errors));
        }

        let parsed = reply.interpret_as_register(register);

        Ok(parsed)
    }

    pub async fn write_register(
        &mut self,
        servo_id: ServoIdOrBroadcast,
        register: Register,
        value: u16,
    ) -> Result<(), ServoBusError> {
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
            .await?
            .map_err(|_| "read failed")?;

        if !reply.servo_status_errors.is_empty() {
            return Err(ServoBusError::ServoStatus(reply.servo_status_errors));
        }

        Ok(())
    }
}
