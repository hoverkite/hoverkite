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
* Misc female-female jumper wires for connecting to the above headers.
* Soldering iron + solder + solder sucker.
* Raspberry Pi (Raspberry Pi 4 is recommended, because it's powerful enough to compile rust projects without taking the age of the universe, but you can do it without this if you have a laptop running Linux).
* [ST-Link V2](https://thepihut.com/products/st-link-stm8-stm32-v2-programmer-emulator) (Don't get the V3MINI. It's a trap).
  * We ended up buying two of these, so we can hack on things independently.
* A pair of serial ports (you can get away with just one if you solder on the extra set of headers onto the master board)
  * PL2303-based USB-Serial port adapters are cheap and super-convenient. You probably want at least one of these in your life.
  * The raspberry pi 4 also has a bunch of pins that can be turned into serial
    ports.
* A game controller for [Hovercontrol](./hovercontrol).


## Build instructions


This section is a bit bare-bones. If you get stuck at any point, follow along with https://hackaday.io/project/170932/instructions and then send a patch to clarify these instructions so they're easier to follow.

* Take the hoverboard apart and solder some headers onto the CPU port of each mainboard, and to the "remote" port on one of the boards (the one without bluetooth, if you have the same Zinc board as us).
  * For us, the "remote" port needed solder-sucking before we could fit the header pins in the holes.
  * In his video, Phil Malone only solders into the "remote" port. This is because he has some header pins attached to his st-link that he can just push-fit into the holes while flashing. You can probably do that too, but we have soldered our headers in place.
* Flash something to disable the GD32 hardware watchdog. Kiel does this automatically on first flash and it stays disabled forever until it is explicitly enabled, or you can do this in openocd `openocd -f interface/stlink-v2.cfg -f target/stm32f1x.cfg -c init -c "reset halt; stm32f1x options_write 0 SWWDG"` according to [this platformio thread](https://community.platformio.org/t/library-for-gd32f130c8/7410/10).
* Run st-util in one tab
  * if you have the stlink plugged into your raspberry pi then you can do:
    `ssh -L4242:localhost:4242 pi@raspberrypi.local st-util` to make things available on your laptop
* run `(cd hoverkite-firmware && cargo run)` in another tab to flash the firmware.
  * You will need gdb-multiarch installed on linux.
  * If you are running on macos then [Ferrous Systems recommend getting an arm-specific gdb from ARM's website instead](https://github.com/ferrous-systems/embedded-trainings/blob/master/INSTALL.md#arm-none-eabi-gdb). You will need to patch `.cargo/config` `runner = "arm-none-eabi-gdb -q -x openocd.gdb"` to reflect this. Shout if you have a better way to do this.
* If you want to flash the firmware while the board is hooked up to the motors then:
  * Connect the battery, motor and power-button wires.
  * Disconnect the 3.3v line of the ST-LINK (I heard a rumour that Bad Things could happen if the ST-LINK and the hoverboard both try to power this line, but I've not tested it. If you think that it's safe to skip this step then please tell me, because the step below is super-annoying and I'd much rather avoid it if it's safe).
  * Hold down the power button while doing `cargo run` (the flasher doesn't latch-up the power, so if you let go you may end up losing power to the board before you're finished flashing).
* Connect to the board via serial port (tx goes to rx, ground goes to ground, and 5v stays disconnected)
* Start the controller software, by following the instructions in [hovercontrol](./hovercontrol). This needs to be run on a device that's connected to the hoverboard via serial port (I recommend running it on the raspberry pi).

## Other resources

* [Hoverboards for Assistive Devices](https://hackaday.io/project/170932-hoverboards-for-assistive-devices) is a really well-documented hardware hacking project that uses these boards.
* The [Field Oriented Control](https://github.com/EmanuelFeru/hoverboard-firmware-hack-FOC) repo is a reasonably active community developing C firmware for single-mainboard hoverboards. Their telegram chat is super-friendly.
* [This platformio thread](https://community.platformio.org/t/library-for-gd32f130c8/7410) in which maxgerhardt ports some Kiel-based hoverboard firmware so that it can be compiled and flashed using platformio.


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
