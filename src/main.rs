use crate::{
    clock::ClockTime,
    render::{AsciiRenderer, ClockRenderer},
};

mod clock;
mod render;

fn main() -> anyhow::Result<()> {
    let (width, height) = crossterm::terminal::size()?;

    println!("Terminal size: ({width}x{height})");

    let renderer = AsciiRenderer::default();

    let output = renderer.render(ClockTime::new(10, 10, 30), width, height);

    println!("{output}");

    Ok(())
}
