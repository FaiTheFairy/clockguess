mod clock;
mod render;

fn main() -> anyhow::Result<()> {
    let (width, height) = crossterm::terminal::size()?;

    println!("Terminal size: ({width}x{height})");
    Ok(())
}
