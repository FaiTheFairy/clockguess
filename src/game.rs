use anyhow::Context;
use crossterm::{
    cursor::MoveTo,
    execute,
    terminal::{Clear, ClearType},
};

use std::io::{BufRead, Write};
use std::time::Instant;

use crate::{AsciiRenderer, Cli, clock::ClockTime, render::ClockRenderer};

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum RoundControl {
    Continue,
    Quit,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum PlayerInput {
    Answer(ClockTime),
    Quit,
}

pub fn run(
    cli: &Cli,
    renderer: &AsciiRenderer,
    input: &mut impl BufRead,
    output: &mut impl Write,
) -> anyhow::Result<()> {
    let mut buffer = String::with_capacity(16);

    match cli.rounds {
        Some(rounds) => {
            for _ in 0..rounds {
                if play_round(&renderer, cli, input, output, &mut buffer)? == RoundControl::Quit {
                    break;
                }
            }
        }
        None => {
            while play_round(&renderer, cli, input, output, &mut buffer)? == RoundControl::Continue
            {
            }
        }
    }
    Ok(())
}

fn play_round(
    renderer: &AsciiRenderer,
    cli: &Cli,
    input: &mut impl BufRead,
    output: &mut impl Write,
    buffer: &mut String,
) -> anyhow::Result<RoundControl> {
    if !cli.no_clear {
        clear_terminal(output)?;
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
    let show_seconds = cli.show_seconds.resolve(difficulty);
    let rendered_clock = renderer.render(expected, width, clock_height, show_seconds);

    writeln!(output, "{rendered_clock}")?;
    writeln!(
        output,
        "What's the time? Read to the {}",
        difficulty.description()
    )?;
    write!(output, "Enter a time such as 12:30, or q to quit: ")?;
    output.flush()?;

    let started = Instant::now();

    let answer = match read_player_input(input, output, buffer)? {
        PlayerInput::Answer(answer) => answer,
        PlayerInput::Quit => return Ok(RoundControl::Quit),
    };

    let elapsed = started.elapsed();
    let difference = expected.analog_difference(answer);

    if difficulty.accepts(expected, answer) {
        writeln!(output)?;

        if difference == 0 {
            writeln!(output, "Correct! Exact answer.")?;
        } else {
            writeln!(
                output,
                "Correct! Your answer was off by {}.",
                format_duration(difference)
            )?;
        }
    } else {
        writeln!(output)?;
        writeln!(
            output,
            "Incorrect. Your answer was off by {}.",
            format_duration(difference)
        )?;
    }

    writeln!(output, "The time was {}", expected.to_12_hour())?;
    writeln!(output, "Answered in {:.1} seconds.", elapsed.as_secs_f32())?;
    writeln!(output)?;
    ask_to_continue(input, output, buffer)
}

fn ask_to_continue(
    input: &mut impl BufRead,
    output: &mut impl Write,
    buffer: &mut String,
) -> anyhow::Result<RoundControl> {
    write!(
        output,
        "Press Enter for another clock, or q then Enter to quit: "
    )?;
    output.flush()?;

    buffer.clear();

    if input.read_line(buffer)? == 0 {
        return Ok(RoundControl::Quit);
    }

    Ok(if buffer.trim().eq_ignore_ascii_case("q") {
        RoundControl::Quit
    } else {
        RoundControl::Continue
    })
}

fn read_player_input(
    input: &mut impl BufRead,
    output: &mut impl Write,
    buffer: &mut String,
) -> anyhow::Result<PlayerInput> {
    loop {
        buffer.clear();

        if input.read_line(buffer)? == 0 {
            return Ok(PlayerInput::Quit);
        }

        let answer = buffer.trim();

        if answer.eq_ignore_ascii_case("q") {
            return Ok(PlayerInput::Quit);
        }

        match answer.parse::<ClockTime>() {
            Ok(time) => return Ok(PlayerInput::Answer(time)),
            Err(error) => {
                writeln!(output, "Invalid time: {error}")?;
                write!(output, "Try again, or enter q to quit: ")?;
                output.flush()?;
            }
        }
    }
}

fn clear_terminal(output: &mut impl Write) -> anyhow::Result<()> {
    execute!(output, Clear(ClearType::All), MoveTo(0, 0))?;
    Ok(())
}

fn format_duration(total_seconds: u32) -> String {
    let minutes = total_seconds / 60;
    let seconds = total_seconds % 60;

    match (minutes, seconds) {
        (0, seconds) => unit(seconds, "second", "seconds"),
        (minutes, 0) => unit(minutes, "minute", "minutes"),
        (minutes, seconds) => format!(
            "{} and {}",
            unit(minutes, "minute", "minutes"),
            unit(seconds, "second", "seconds")
        ),
    }
}

fn unit(value: u32, singular: &str, plural: &str) -> String {
    let label = if value == 1 { singular } else { plural };
    format!("{value} {label}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn q_quits_during_answer_prompt() {
        let mut input = "q\n".as_bytes();
        let mut output = Vec::new();
        let mut buffer = String::new();

        let result = read_player_input(&mut input, &mut output, &mut buffer).unwrap();

        assert!(matches!(result, PlayerInput::Quit));
    }

    #[test]
    fn eof_quits_during_answer_prompt() {
        let mut input = "".as_bytes();
        let mut output = Vec::new();
        let mut buffer = String::new();

        let result = read_player_input(&mut input, &mut output, &mut buffer).unwrap();

        assert!(matches!(result, PlayerInput::Quit));
    }

    #[test]
    fn invalid_input_is_retried() {
        let mut input = "invalid\n9:30\n".as_bytes();
        let mut output = Vec::new();
        let mut buffer = String::new();

        let result = read_player_input(&mut input, &mut output, &mut buffer).unwrap();

        assert!(matches!(
            result,
            PlayerInput::Answer(time) if time == ClockTime::new(9, 30, 0)
        ));

        let output = String::from_utf8(output).unwrap();
        assert!(output.contains("Invalid time:"));
        assert!(output.contains("Try again"));
    }

    #[test]
    fn eof_does_not_continue() {
        let mut input = "".as_bytes();
        let mut output = Vec::new();
        let mut buffer = String::new();

        let result = ask_to_continue(&mut input, &mut output, &mut buffer).unwrap();

        assert!(result == RoundControl::Quit);
    }

    #[test]
    fn test_fromat_duration() {
        assert_eq!(format_duration(0), "0 seconds");
        assert_eq!(format_duration(1), "1 second");
        assert_eq!(format_duration(60), "1 minute");
        assert_eq!(format_duration(61), "1 minute and 1 second");
    }
}
