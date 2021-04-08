use core::{convert::TryInto, ops::RangeInclusive};

pub fn clamp<T: PartialOrd + TryInto<B>, B: Into<T> + Copy>(x: T, range: &RangeInclusive<B>) -> B {
    if x > (*range.end()).into() {
        *range.end()
    } else if x < (*range.start()).into() {
        *range.start()
    } else {
        x.try_into().ok().unwrap()
    }
}
