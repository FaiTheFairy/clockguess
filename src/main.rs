use std::{
    io::{self, Write},
    time::{Duration, Instant},
};

use anyhow::Context;

use crate::{
    clock::ClockTime,
    render::{AsciiRenderer, ClockRenderer},
};

use crossterm::{
    cursor::MoveTo,
    execute,
    terminal::{Clear, ClearType},
};

mod clock;
mod render;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum Difficulty {
    NearestHour,
    NearestFiveMinutes,
    NearestMinute,
    NearestThirtySeconds,
    Exact,
}

impl Difficulty {
    /// The interval to which the player is expected to read the clock
    fn precision_seconds(self) -> u32 {
        match self {
            Difficulty::NearestHour => 60 * 60,
            Difficulty::NearestFiveMinutes => 5 * 60,
            Difficulty::NearestMinute => 60,
            Difficulty::NearestThirtySeconds => 30,
            Difficulty::Exact => 0,
        }
    }

    /// An answer rounded to an interval may be at most half an interval away
    fn tolerance_seconds(self) -> u32 {
        self.precision_seconds() / 2
    }

    fn description(self) -> &'static str {
        match self {
            Difficulty::NearestHour => "nearest hour",
            Difficulty::NearestFiveMinutes => "nearest five minutes",
            Difficulty::NearestMinute => "nearest minute",
            Difficulty::NearestThirtySeconds => "nearest thirty seconds",
            Difficulty::Exact => "exact time",
        }
    }

    fn accepts(self, expected: ClockTime, answer: ClockTime) -> bool {
        expected.analog_difference(answer) <= self.tolerance_seconds()
    }
}

enum PlayerInput {
    Answer(ClockTime),
    Quit,
}

fn main() -> anyhow::Result<()> {
    let renderer = AsciiRenderer::default();
    let difficulty = Difficulty::NearestFiveMinutes;

    let stdin = io::stdin();
    let mut stdout = io::stdout();
    let mut input = String::with_capacity(16);

    loop {
        let should_continue = play_round(&renderer, difficulty, &stdin, &mut stdout, &mut input)?;

        if !should_continue {
            break;
        }
    }
    Ok(())
}

fn play_round(
    renderer: &AsciiRenderer,
    difficulty: Difficulty,
    stdin: &io::Stdin,
    stdout: &mut io::Stdout,
    input: &mut String,
) -> anyhow::Result<bool> {
    clear_terminal(stdout)?;

    let (width, height) = crossterm::terminal::size()?;
    let clock_height = height.saturating_sub(4);

    let expected = ClockTime::random();
    let rendered_clock = renderer.render(expected, width, clock_height);

    writeln!(stdout, "{rendered_clock}")?;
    writeln!(
        stdout,
        "What's the time? Read to the {}",
        difficulty.description()
    )?;
    writeln!(stdout, "Enter a time such as 12:30, or q to quit: ")?;
    stdout.flush()?;

    let started = Instant::now();

    let answer = match read_player_input(stdin, stdout, input)? {
        PlayerInput::Answer(answer) => answer,
        PlayerInput::Quit => return Ok(false),
    };

    let elapsed = started.elapsed();
    let difference = expected.analog_difference(answer);

    if difficulty.accepts(expected, answer) {
        writeln!(stdout)?;
        writeln!(
            stdout,
            "Correct!, Your answer was off by {}.",
            format_duration(difference)
        )?;
    } else {
        writeln!(stdout)?;
        writeln!(
            stdout,
            "Incorrect. Your answer was off by {}.",
            format_duration(difference)
        )?;
    }

    writeln!(stdout, "The time was {}", expected.to_12_hour())?;
    writeln!(stdout, "Answered in {:.1} seconds.", elapsed.as_secs_f32())?;
    writeln!(stdout)?;
    write!(
        stdout,
        "Press Enter for another clock, or q then enter to quit"
    )?;
    stdout.flush()?;

    input.clear();
    stdin.read_line(input)?;

    Ok(!input.trim().eq_ignore_ascii_case("q"))
}

fn read_player_input(
    stdin: &io::Stdin,
    stdout: &mut io::Stdout,
    input: &mut String,
) -> anyhow::Result<PlayerInput> {
    loop {
        input.clear();

        if stdin.read_line(input)? == 0 {
            return Ok(PlayerInput::Quit);
        }

        let input = input.trim();

        if input.eq_ignore_ascii_case("q") {
            return Ok(PlayerInput::Quit);
        }

        match input.parse::<ClockTime>() {
            Ok(time) => return Ok(PlayerInput::Answer(time)),
            Err(error) => {
                writeln!(stdout, "Invalid time: {error}")?;
                write!(stdout, "Try again, or enter q to quit: ")?;
                stdout.flush()?;
            }
        }
    }
}

fn clear_terminal(stdout: &mut io::Stdout) -> anyhow::Result<()> {
    execute!(stdout, Clear(ClearType::All), MoveTo(0, 0))?;
    Ok(())
}

fn format_duration(total_seconds: u32) -> String {
    let minutes = total_seconds / 60;
    let seconds = total_seconds % 60;

    match (minutes, seconds) {
        (0, seconds) => format!("{seconds} seconds"),
        (minutes, 0) => format!("{minutes} minutes"),
        (minutes, seconds) => format!("{minutes} minutes and {seconds} seconds"),
    }
}
