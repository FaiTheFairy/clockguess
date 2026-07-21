use std::time::Duration;

use anyhow::Context;

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
    let mut input = String::with_capacity(5);

    loop {
        let time = ClockTime::random();

        let output = renderer.render(time, width, height - 2);
        println!("{output}");
        println!("What's the time? e.g. '12:30'");
        let now = std::time::Instant::now();

        std::io::stdin().read_line(&mut input)?;

        let (hours, minutes) = input.trim().split_once(':').context("time missing ':'")?;
        let hours: u8 = hours.parse()?;
        let minutes = minutes.parse()?;

        if hours == time.hour_12() && ((time.minute() - 5)..(time.minute() + 5)).contains(&minutes)
        {
            let elapsed = now.elapsed();
            println!(
                "Correct! off by {} minutes",
                time.minute().abs_diff(minutes)
            );
            println!("Answered in {}s", elapsed.as_secs_f32())
        } else {
            println!("Incorrect! time is {}:{}", time.hour_12(), time.minute());
        }

        input.clear();
        std::thread::sleep(Duration::from_secs(2));
    }

    Ok(())
}
