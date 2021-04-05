use eyre::Report;
use gilrs::{Axis, Button, Event, EventType, Gilrs};

fn main() -> Result<(), Report> {
    stable_eyre::install()?;
    pretty_env_logger::init();
    color_backtrace::install();

    let mut gilrs = Gilrs::new().unwrap();

    let mut offset_left: i64 = 0;
    let mut offset_right: i64 = 0;
    let mut centre_left: i64 = 0;
    let mut centre_right: i64 = 0;
    let mut scale = 20.0;
    let centre_step = 10;

    loop {
        while let Some(Event {
            id: _,
            event,
            time: _,
        }) = gilrs.next_event()
        {
            match event {
                EventType::AxisChanged(Axis::LeftStickY, value, _code) => {
                    offset_left = (scale * value) as i64;
                    set_target(Side::Left, centre_left + offset_left);
                }
                EventType::AxisChanged(Axis::RightStickY, value, _code) => {
                    offset_right = (scale * value) as i64;
                    set_target(Side::Right, centre_right + offset_right);
                }
                EventType::ButtonPressed(Button::DPadLeft, _code) => {
                    if scale > 1.0 {
                        scale -= 1.0;
                    }
                    println!("Scale {}", scale);
                }
                EventType::ButtonPressed(Button::DPadRight, _code) => {
                    if scale < 100.0 {
                        scale += 1.0;
                    }
                    println!("Scale {}", scale);
                }
                EventType::ButtonPressed(Button::LeftTrigger, _code) => {
                    centre_left += centre_step;
                    set_target(Side::Left, centre_left + offset_left);
                }
                EventType::ButtonPressed(Button::LeftTrigger2, _code) => {
                    centre_left -= centre_step;
                    set_target(Side::Left, centre_left + offset_left);
                }
                EventType::ButtonPressed(Button::RightTrigger, _code) => {
                    centre_right += centre_step;
                    set_target(Side::Right, centre_right + offset_right);
                }
                EventType::ButtonPressed(Button::RightTrigger2, _code) => {
                    centre_right -= centre_step;
                    set_target(Side::Right, centre_right + offset_right);
                }
                EventType::ButtonPressed(button, code) => {
                    println!("Button {:?} pressed: {:?}", button, code);
                }
                _ => {}
            }
        }
    }
}

fn set_target(side: Side, target: i64) {
    println!("Target {:?} {}", side, target);
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
enum Side {
    Left,
    Right,
}
