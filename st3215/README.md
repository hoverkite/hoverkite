# ST3215

A crate for communicating with the Feetech/Waveshare branded
[https://www.waveshare.com/wiki/ST3215_Servo](ST3215 serial bus servo).

It is still very much a work in progress. Going forward, it might be worth emulating one of the
existing dynamixel servo bus crates (e.g. https://crates.io/keywords/dynamixel or
https://github.com/pollen-robotics/rustamixel).
This would allow us to have a consistent API for programming different servo-based robot arms.
Eventually, it might be nice create a single set of traits (or even structs) that work for both
dynamixel and ST3215. For now though, I am taking the "innovate then standardize" approach,
and completely ignoring the structure of existing servo bus crates.

The core of this crate is written in a sans-io style, so it **should** work with any blocking/async
serial port implementation on Windows/MacOS/Linux/embedded/no_std.

The primary user of this crate is the hoverkite project, for the ESP32 firmware of our kitebox
kite control robot.
This means that embassy + ESP32 on the Waveshare
[https://www.waveshare.com/wiki/General_Driver_for_Robots](General Driver For Robotics) dev board is
the most well tested target for this library.
