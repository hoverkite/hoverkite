#[derive(Debug, Clone, Copy)]
pub enum TtyCommand {
    Ping,
    Up,
    Down,
    Left,
    Right,
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
            b'p' => Self::Ping,
            b'^' => Self::Up,
            b'v' => Self::Down,
            b'<' => Self::Left,
            b'>' => Self::Right,
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
            other => Self::Unrecognised(other),
        })
    }

    // TODO: proptest that TtyCommand::read_async([cmd.as_u8()]).await == cmd for all cmd?
    pub fn as_u8(&self) -> u8 {
        match self {
            TtyCommand::Ping => b'p',
            TtyCommand::Up => b'^',
            TtyCommand::Down => b'v',
            TtyCommand::Left => b'<',
            TtyCommand::Right => b'>',
            TtyCommand::Unrecognised(_) => b'?',
        }
    }
}
