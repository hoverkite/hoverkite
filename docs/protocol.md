# Protocol

## Commands

Each command sent from the controller to the hoverboard consists of a single ASCII character
defining the command, followed by some number of bytes of parameters. The number of parameter bytes
depends on the command.

| Command | Parameters | Meaning                                                 |
| ------- | ---------- | ------------------------------------------------------- |
| l       | '0' or '1' | Turn side LEDs on or off.                               |
| o       | '0' or '1' | Turn orange LED on or off.                              |
| r       | '0' or '1' | Turn red LED on or off.                                 |
| g       | '0' or '1' | Turn green LED on or off.                               |
| b       | none       | Dump battery voltages.                                  |
| c       | none       | Dump whether charger is connected.                      |
| S       | i16, i16   | Set maximum speed (negative and positive).              |
| K       | u16        | Set spring constant.                                    |
| n       | none       | Remove target position.                                 |
| T       | i64        | Set target position.                                    |
| e       | none       | Set current position as 0 position and target position. |
| p       | none       | Power off.                                              |
| F       | u8, \*     | Forward the following N bytes to the other side.        |

## Responses

A response from the hoverboard to the controller similarly consists of a single ASCII character
followed by some number of bytes of parameters.

| Response | Parameters       | Meaning                                                                         |
| -------- | ---------------- | ------------------------------------------------------------------------------- |
| "        | Up until newline | Log message                                                                     |
| '        | Up until newline | Log message forwarded from secondary                                            |
| I        | i64              | Current position update                                                         |
| i        | i64              | Current position update forwarded from secondary                                |
| B        | u16, u16, u16    | Battery voltage, backup battery voltage, motor current                          |
| b        | u16, u16, u16    | Battery voltage, backup battery voltage, motor current forwarded from secondary |
| p        | none             | Power off.                                                                      |
