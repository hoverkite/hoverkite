# ST3215

A crate for communicating with Feetech/Waveshare branded
[https://www.waveshare.com/wiki/ST3215_Servo](ST3215 serial bus servos).

It is still very much a "works for me in hoverkite" level of maturity.
Going forward, it might be worth [https://github.com/pollen-robotics/rustypot/issues/66](adding support for ST3215 servos to rustypot)
and then fixing their codegen macros so that they also produce async code behind a feature flag.
This would allow me to use their crate and deprecate this one. It would also give people a lot more
choice about which servos they use when replicating my projects.
For now though, I am taking the "innovate then standardize" approach, and completely ignoring the
structure of existing servo bus crates.

The core of this crate is written in a sans-io style, so it **should** work with any blocking or
async serial port implementation on Windows/MacOS/Linux/embedded/no_std.

The primary user of this crate is the hoverkite project, for the ESP32 firmware of our kitebox
kite control robot.
This means that embassy + ESP32 on the Waveshare
[https://www.waveshare.com/wiki/General_Driver_for_Robots](General Driver For Robotics) dev board is
the most well tested target for this library.
