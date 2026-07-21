use std::time::Duration;

use anyhow::Context;

use crate::{
    clock::ClockTime,
    render::{AsciiRenderer, ClockRenderer},
};

mod clock;
mod render;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum Difficulty {
    NearestHour,
    NearestFiveMin,
    NearestMin,
    NearestThirtySec,
    Exact,
}

impl Difficulty {
    fn seconds_delta(self) -> u32 {
        match self {
            Difficulty::NearestHour => 60 * 60,
            Difficulty::NearestFiveMin => 60 * 5,
            Difficulty::NearestMin => 60,
            Difficulty::NearestThirtySec => 30,
            Difficulty::Exact => 0,
        }
    }
}

fn main() -> anyhow::Result<()> {
    let renderer = AsciiRenderer::default();
    let mut input = String::with_capacity(6);
    let difficulty = Difficulty::NearestFiveMin;

    loop {
        let (width, height) = crossterm::terminal::size()?;

        let time = ClockTime::random();
        let output = renderer.render(time, width, height.saturating_sub(2));

        println!("{output}");
        println!("What's the time? For example, 12:30");

        let started = std::time::Instant::now();

        input.clear();
        std::io::stdin().read_line(&mut input)?;

        let answer_time: ClockTime = input.parse()?;

        const SECONDS_PER_CYCLE: u32 = 12 * 60 * 60;

        let expected_seconds = time.total_seconds() % SECONDS_PER_CYCLE;
        let answer_seconds = answer_time.total_seconds() % SECONDS_PER_CYCLE;

        let difference = expected_seconds.abs_diff(answer_seconds);
        let difference = difference.min(SECONDS_PER_CYCLE - difference);

        if difference <= difficulty.seconds_delta() {
            let elapsed = started.elapsed();
            println!(
                "Correct! off by {} minutes\nTime is {time}",
                difference as f64 / 60.0
            );
            println!("Answered in {}s", elapsed.as_secs_f32())
        } else {
            println!("Incorrect! time is {time}");
        }

        input.clear();
        std::thread::sleep(Duration::from_secs(2));
    }

    Ok(())
}
