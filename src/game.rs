use anyhow::Context;
use crossterm::{
    cursor::MoveTo,
    execute,
    terminal::{Clear, ClearType},
};

use std::io;
use std::io::Write;
use std::time::Instant;

use crate::AsciiRenderer;
use crate::Cli;
use crate::clock::ClockTime;
use crate::render::ClockRenderer;

enum PlayerInput {
    Answer(ClockTime),
    Quit,
}

pub fn run(
    cli: &Cli,
    renderer: &AsciiRenderer,
    stdin: io::Stdin,
    mut stdout: io::Stdout,
) -> Result<(), anyhow::Error> {
    let mut input = String::with_capacity(16);

    match cli.rounds {
        Some(rounds) => {
            for _ in 0..rounds {
                if !play_round(&renderer, &cli, &stdin, &mut stdout, &mut input)? {
                    break;
                }
            }
        }
        None => while play_round(&renderer, &cli, &stdin, &mut stdout, &mut input)? {},
    }
    Ok(())
}

fn play_round(
    renderer: &AsciiRenderer,
    cli: &Cli,
    stdin: &io::Stdin,
    stdout: &mut io::Stdout,
    input: &mut String,
) -> anyhow::Result<bool> {
    if !cli.no_clear {
        clear_terminal(stdout)?;
    }

    let difficulty = cli.difficulty;

    let (width, height) = crossterm::terminal::size()?;

    // override with cli width and height if given
    let width = cli.width.unwrap_or(width);
    let height = cli.height.unwrap_or(height);

    let clock_height = height
        .checked_sub(4)
        .context("terminal is too short to display the clock; increase its height")?;

    let expected = ClockTime::random();
    let rendered_clock = renderer.render(expected, width, clock_height, difficulty.hide_seconds());

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
    ask_to_continue(stdin, stdout, input)
}

fn ask_to_continue(
    stdin: &io::Stdin,
    stdout: &mut io::Stdout,
    input: &mut String,
) -> Result<bool, anyhow::Error> {
    write!(
        stdout,
        "Press Enter for another clock, or q then enter to quit"
    )?;
    stdout.flush()?;

    input.clear();

    if stdin.read_line(input)? == 0 {
        return Ok(false);
    }

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
