# Hovercontrol

Hovercontrol is a utility for controlling the Hoverkite with a game controller. It expects a serial
port connected to each side of the hoverboard, and a game controller supported by the OS.

## Usage

Create a config file called `hovercontrol.toml`, with at least a `right_port` set. See
[`hovercontrol.example.toml`](hovercontrol.example.toml) for details.

Then you can run it like:

```shell
$ cargo run
```

## Control mapping

| Control            | Usage                                      |
| ------------------ | ------------------------------------------ |
| Left stick Y axis  | Left motor offset                          |
| Right stick Y axis | Right motor offset                         |
| Left stick button  | Set current position as left motor centre  |
| Right stick button | Set current position as right motor centre |
| D-pad left         | Decrease stick scale                       |
| D-pad right        | Incease stick scale                        |
| D-pad down         | Decrease max torque/speed                  |
| D-pad up           | Increase max torque/speed                  |
| L1                 | Increase left motor centre                 |
| L2                 | Decrease left motor centre                 |
| R1                 | Increase right motor centre                |
| R2                 | Decrease right motor centre                |
| A                  | Dump battery state                         |
| B                  | Remove target for both motors              |
| X                  | Decrease spring constant                   |
| Y                  | Increase spring constant                   |
| Mode               | Power off                                  |

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
