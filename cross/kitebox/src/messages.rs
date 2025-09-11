use kitebox_messages::Command;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TtyCommand {
    Newline,
    Ping,
    Up,
    Down,
    Left,
    Right,
    Release,
    Query,
    Binary(Command),
    // FIXME: make this into an error instead?
    Unrecognised(u8),
}
impl TtyCommand {
    pub async fn read_async<R: embedded_io_async::Read>(
        mut stream: R,
    ) -> Result<Self, embedded_io_async::ReadExactError<R::Error>> {
        let mut buffer = [0u8; 1];
        stream.read_exact(&mut buffer).await?;

        Ok(match buffer[0] {
            b'\n' => Self::Newline,
            b'p' => Self::Ping,
            b'^' => Self::Up,
            b'v' => Self::Down,
            b'<' => Self::Left,
            b'>' => Self::Right,
            b'r' => Self::Release,
            b'?' => Self::Query,
            27 => {
                // Escape codes. Used by arrow keys.
                stream.read_exact(&mut buffer).await?;
                match buffer[0] {
                    b'[' => {
                        stream.read_exact(&mut buffer).await?;
                        match buffer[0] {
                            b'A' => Self::Up,
                            b'B' => Self::Down,
                            b'D' => Self::Left,
                            b'C' => Self::Right,
                            other => Self::Unrecognised(other),
                        }
                    }
                    other => Self::Unrecognised(other),
                }
            }
            b'#' => {
                // Read a COBS-encoded postcard message
                let mut buffer = [0u8; kitebox_messages::MAX_MESSAGE_SIZE];
                let mut idx = 0;

                // Read until we get a zero byte (COBS terminator)
                loop {
                    if idx >= buffer.len() {
                        // Message too long - skip until terminator
                        loop {
                            stream.read_exact(&mut buffer[..1]).await?;
                            if buffer[0] == 0 {
                                return Ok(Self::Unrecognised(b'L')); // L for too Long
                            }
                        }
                    }
                    stream.read_exact(&mut buffer[idx..idx + 1]).await?;
                    if buffer[idx] == 0 {
                        break;
                    }
                    idx += 1;
                }

                // Try to decode the command
                match Command::from_slice(&mut buffer[..idx]) {
                    Ok(cmd) => Self::Binary(cmd),
                    Err(e) => {
                        log::error!("Failed to decode command message, {e}");
                        Self::Unrecognised(b'F')
                    }
                }
            }
            other => Self::Unrecognised(other),
        })
    }

    // FIXME: kill this off?
    pub fn as_u8(&self) -> u8 {
        match self {
            TtyCommand::Newline => b'\n',
            TtyCommand::Ping => b'p',
            TtyCommand::Up => b'^',
            TtyCommand::Down => b'v',
            TtyCommand::Left => b'<',
            TtyCommand::Right => b'>',
            TtyCommand::Release => b'r',
            TtyCommand::Query => b'?',
            TtyCommand::Binary(_) => b'#',
            TtyCommand::Unrecognised(_) => b'?',
        }
    }
}
