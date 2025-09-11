use esp_println::println;
use kitebox_messages::{Command, CommandMessage};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TtyCommand {
    Newline,
    Ping,
    Up,
    Down,
    Left,
    Right,
    Query,
    Capnp(Command),
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
                // FIXME: DRY
                // The following bytes are a capnproto message, using the recommended
                // serialization scheme from
                // https://capnproto.org/encoding.html#serialization-over-a-stream
                let mut buf = [0u8; 4];
                // N segments - 1 should always be 0 for a SingleSegmentAllocator
                stream.read_exact(&mut buf).await.unwrap();
                assert_eq!(u32::from_le_bytes(buf), 0);

                // FIXME: fuzz this. It might be possible to drop into the middle of a
                // message and interpret it as a message with a huge length, then wait
                // forever for the esp32 to actually send us that much data.
                stream.read_exact(&mut buf).await.unwrap();
                let len = u32::from_le_bytes(buf) as usize;
                let mut buf = [0u8; CommandMessage::SEGMENT_ALLOCATOR_SIZE];
                stream.read_exact(&mut buf[..len]).await.unwrap();
                let slice = &buf[..len];

                match CommandMessage::from_slice(&slice) {
                    Ok(message) => Self::Capnp(message.command),
                    Err(e) => {
                        println!("error decoding message: {e:?}");
                        // skip until the next newline or #. I kind-of wish we were using cobs
                        // or something for our payloading so that recovering was easier.
                        // FIXME: BufRead::skip_until() or something?
                        loop {
                            let mut buf = [0u8];
                            stream.read_exact(&mut buf).await.unwrap();
                            if let b'\n' | b'#' = buf[0] {
                                break;
                            }
                        }
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
            TtyCommand::Query => b'?',
            TtyCommand::Capnp(_) => b'#',
            TtyCommand::Unrecognised(_) => b'?',
        }
    }
}
