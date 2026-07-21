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

        let expected_minutes = time.total_seconds();
        let answer_minutes = answer_time.total_seconds();
        let difference = expected_minutes.abs_diff(answer_minutes);
        const SECONDS_PER_CYCLE: u32 = 12 * 60 * 60;
        let difference = difference.min(SECONDS_PER_CYCLE - difference);

        if difference <= difficulty.seconds_delta() {
            let elapsed = started.elapsed();
            println!(
                "Correct! off by {} minutes",
                time.minute().abs_diff(answer_time.minute())
            );
            println!("Answered in {}s", elapsed.as_secs_f32())
        } else {
            println!("Incorrect! time is {}:{:02}", time.hour_12(), time.minute());
        }

        input.clear();
        std::thread::sleep(Duration::from_secs(2));
    }

    Ok(())
}
