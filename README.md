# HoverKite

HoverKite is a hare-brained scheme to fly a kite using a camera, a raspberry pi, and custom firmware
on a hoverboard.

Progress is tracked on [Trello](https://trello.com/b/v4vMHzf9/kite-power-generation). Eventually we
would like to use this setup to produce power, but that's a long way off.

There are currently two crates in this repository:

- [Firmware](./hoverkite-firmware) for a hoverboard.
- [A utility](./hovercontrol) to control it with a game controller.

They communicate over a serial port using a custom [protocol](docs/protocol.md).

## Getting started

You will need:

* A hoverboard with a pair of `TT-SD2.2` main-boards ("Split Board" according to [Candas' spreadsheet](https://docs.google.com/spreadsheets/d/1Vs3t2W8_z7E93Ij0pwx_cKzxzKSvjW1n1t_40CXV3ik/edit#gid=0))
  * This project is being developed on a `Zinc Smart GT Pro`.
  * https://hackaday.io/project/170932-hoverboards-for-assistive-devices suggests that `HOVER-1 ULTRA Hoverboard` is another example of such a hoverboard.
  * Please send a patch if you know of other split-mainboard hoverboards that are likely to be compatible.
* 2.54mm (0.1") headers (you need to make 2-3 headers, each with 4 pins).
* Soldering iron + solder + solder sucker.
* Raspberry Pi (Raspberry Pi 4 is recommended, because it's powerful enough to compile rust projects without taking the age of the universe, but you can do it without this if you have a laptop running Linux).
* [ST-Link V2](https://thepihut.com/products/st-link-stm8-stm32-v2-programmer-emulator) (Don't get the V3MINI. It's a trap).
  * We ended up buying two of these, so we can hack on things independently.
* A pair of serial ports (you can get away with just one if you solder on the extra set of headers onto the master board)
  * PL2303-based USB-Serial port adapters are cheap and super-convenient. You probably want at least one of these in your life.
  * The raspberry pi 4 also has a bunch of pins that can be turned into serial
    ports.
* A game controller for [Hovercontrol](./hovercontrol).


## Other resources

* [Hoverboards for Assistive Devices](https://hackaday.io/project/170932-hoverboards-for-assistive-devices) is a really well-documented hardware hacking project that uses these boards.
* The [Field Oriented Control](https://github.com/EmanuelFeru/hoverboard-firmware-hack-FOC) repo is a reasonably active community developing C firmware for single-mainboard hoverboards. Their telegram chat is super-friendly.


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
