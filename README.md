# HoverKite

HoverKite is a hare-brained scheme to fly a kite using a camera, a raspberry pi, and custom firmware
on a hoverboard.

Progress is tracked on [Trello](https://trello.com/b/v4vMHzf9/kite-power-generation). Eventually we
would like to use this setup to produce power, but that's a long way off.

There are currently two crates in this repository:

- [Firmware](./hoverkite-firmware) for a hoverboard.
- [A utility](./hovercontrol) to control it with a game controller.

They communicate over a serial port using a custom [protocol](docs/protocol.md).

## License

Licensed under either of

- Apache License, Version 2.0
  ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license
  ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the
work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any
additional terms or conditions.
