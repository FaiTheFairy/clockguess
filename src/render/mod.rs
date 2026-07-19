use crate::clock::ClockTime;

pub trait ClockRenderer {
    type Output;

    fn render(&self, time: ClockTime, width: u16, height: u16) -> Self::Output;
}

mod ascii;
mod canvas;

pub use ascii::AsciiRenderer;
