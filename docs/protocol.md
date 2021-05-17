# Protocol

All numeric values are sent in little-endian order.

## Commands

Each command sent from the controller to the hoverboard consists of the ASCII character 'R' or 'L'
to indicate whether it is for the left or right side, followed by a single ASCII character defining
the command, followed by some number of bytes of parameters. The number of parameter bytes depends
on the command.

| Command | Parameters | Meaning                                                 |
| ------- | ---------- | ------------------------------------------------------- |
| l       | '0' or '1' | Turn side LEDs on or off.                               |
| o       | '0' or '1' | Turn orange LED on or off.                              |
| r       | '0' or '1' | Turn red LED on or off.                                 |
| g       | '0' or '1' | Turn green LED on or off.                               |
| f       | u32        | Set buzzer frequency (or 0 for off).                    |
| b       | none       | Dump battery voltages.                                  |
| c       | none       | Dump whether charger is connected.                      |
| S       | i16, i16   | Set maximum speed (negative and positive).              |
| K       | u16        | Set spring constant.                                    |
| n       | none       | Remove target position.                                 |
| T       | i64        | Set target position.                                    |
| e       | none       | Set current position as 0 position and target position. |
| p       | none       | Power off.                                              |

## Responses

A response from the hoverboard to the controller similarly consists of the ASCII character 'R' or
'L' to indicate whether it is from the left or right side, followed by a single ASCII character
defining the command, followed by some number of bytes of parameters.

| Response | Parameters       | Meaning                                                |
| -------- | ---------------- | ------------------------------------------------------ |
| "        | Up until newline | Log message                                            |
| I        | i64              | Current position update                                |
| B        | u16, u16, u16    | Battery voltage, backup battery voltage, motor current |
| C        | '0' or '1'       | Charger connected                                      |
| p        | none             | Power off (command from secondary to primary).         |
