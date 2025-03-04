# Kitebox firmware for hoverkite

This folder contains ESP32 firmware for a kite controller that lives in the sky and a ground station.

This is inspired by the "soft kite variant" of the kitesforfuture.be project. See slide 13 of [this slide deck](https://www.kitesforfuture.de/FOSDEMSlides.pdf), or [this youtube video](https://www.youtube.com/watch?v=9IuRIYftyb0).

The intention is that we ship exactly the same code to both the ground station and the box in
the sky. The ground kitebox might be connected to a computer over usb, but not connected to a
servo bus, but that's okay: the algorithm is still:

* if you receive anything from tty_uart:
  * attempt to forward it over esp now
  * attempt to action it via the servo_uart
* if you receive anything from esp now:
  * attempt to log it over tty_uart (or in practice esp_println::println!() for now)
  * attempt to action it via the servo_uart
* if any of your attempts fail because there is nothing connected
  * that's fine
  * maybe we can log it later, or add metrics?

```
             ground kitebox                       sky kitebox
            ┌─────────────────────────┐          ┌─────────────────────────┐
            │                         │          │                         │
            │           esp <•••••••••••••••••••••••••••••• esp            │
            │           now ──────────────────────────────► now            │
            │           : ▲           │          │          │ ^            │
            │           : │           │          │          │ :            │
            │           v │           │          │          ▼ :            │
            │    ┌──► main_loop() ••  │          │    ••> main_loop() ─┐   │
            │    │                 v  │          │    :                ▼   │
        usb │               servo_uart••>        │               servo_uart┼────►
         ───►tty_uart                 │        ••>tty_uart                 │ servo
            └─────────────────────────┘          └─────────────────────────┘  bus

             ──► = active connection        ••> = unused connection
```

This is a very similar approach to hoverkite-firmware, but the hoverkite boards are almost
completely identical, and I'm working with a mishmash of esp32 devboards.
I suspect that the approach will start to fall apart when we add accelerometer-based inputs and
sdcard-based logs.
