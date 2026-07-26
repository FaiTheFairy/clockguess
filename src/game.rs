use std::io::{BufRead, Write};
use std::time::{Duration, Instant};

use anyhow::Context;
use crossterm::{
    cursor::MoveTo,
    execute,
    terminal::{Clear, ClearType},
};

use crate::cli::GameMode;
use crate::score::{RoundOutcome, ScoreRecord, ScoreStore, SessionStats};
use crate::{
    cli::Cli,
    clock::{ClockTime, SECONDS_PER_MINUTE},
    render::{AsciiRenderer, ClockRenderer},
};

const UI_ROWS: u16 = 4;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum RoundControl {
    Completed(RoundOutcome),
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
    let score_store = ScoreStore::try_new()?;

    if cli.print_scores {
        return score_store.pretty_write_scores(output);
    }

    let game_mode = cli.mode;
    let stats = match game_mode {
        GameMode::Practice => run_practice(cli, renderer, input, output, &mut buffer)?,
        GameMode::Challenge => run_challenge(cli, renderer, input, output, &mut buffer)?,
        GameMode::RapidFire => run_rapid_fire(cli, renderer, input, output, &mut buffer)?,
    };

    print_summary(output, &stats)?;

    let should_save = game_mode != GameMode::Practice && !stats.has_quit();
    if should_save {
        let record = ScoreRecord::from_session(cli, &stats);
        score_store.add(record)?;
    }
    Ok(())
}

fn run_practice(
    cli: &Cli,
    renderer: &AsciiRenderer,
    input: &mut impl BufRead,
    output: &mut impl Write,
    buffer: &mut String,
) -> anyhow::Result<SessionStats> {
    let mut stats = SessionStats::default();

    loop {
        match play_round(renderer, cli, input, output, buffer)? {
            RoundControl::Completed(outcome) => {
                stats.record(outcome, cli.difficulty);
            }
            RoundControl::Quit => {
                stats.quit();
                break;
            }
        }

        if ask_to_continue(input, output, buffer)? == ContinueChoice::Quit {
            break;
        }
    }

    Ok(stats)
}

fn run_challenge(
    cli: &Cli,
    renderer: &AsciiRenderer,
    input: &mut impl BufRead,
    output: &mut impl Write,
    buffer: &mut String,
) -> anyhow::Result<SessionStats> {
    let mut stats = SessionStats::default();

    for round in 1..=cli.rounds {
        writeln!(output, "Round {round}/{}", cli.rounds)?;

        match play_round(renderer, cli, input, output, buffer)? {
            RoundControl::Completed(outcome) => {
                stats.record(outcome, cli.difficulty);
            }
            RoundControl::Quit => {
                stats.quit();
                break;
            }
        }
    }

    Ok(stats)
}

fn run_rapid_fire(
    cli: &Cli,
    renderer: &AsciiRenderer,
    input: &mut impl BufRead,
    output: &mut impl Write,
    buffer: &mut String,
) -> anyhow::Result<SessionStats> {
    let mut stats = SessionStats::default();

    let duration = Duration::from_secs(cli.rapid_seconds);
    let started = Instant::now();
    let deadline = started + duration;

    while Instant::now() < deadline {
        let remaining = deadline.saturating_duration_since(Instant::now());

        writeln!(
            output,
            "Time remaining: {:.1} seconds",
            remaining.as_secs_f64()
        )?;

        match play_round(renderer, cli, input, output, buffer)? {
            RoundControl::Completed(outcome) => {
                stats.record(outcome, cli.difficulty);
            }
            RoundControl::Quit => {
                stats.quit();
                break;
            }
        }
    }

    if stats.has_quit() {
        writeln!(output, "Session ended.")?;
    } else {
        writeln!(output, "Time's up!")?;
    }

    Ok(stats)
}

fn print_summary(output: &mut impl Write, stats: &SessionStats) -> anyhow::Result<()> {
    writeln!(output)?;
    writeln!(output, "Session complete")?;
    writeln!(output, "----------------")?;
    writeln!(output, "Score: {}", stats.points())?;
    writeln!(output, "Correct: {}", stats.correct())?;
    writeln!(output, "Incorrect: {}", stats.incorrect())?;
    writeln!(output, "Exact answers: {}", stats.exact())?;

    match stats.accuracy() {
        Some(accuracy) => writeln!(output, "Accuracy: {:.1}%", accuracy * 100.0)?,
        None => writeln!(output, "Accuracy: N/A")?,
    }

    match stats.average_answer_time() {
        Some(average) => writeln!(
            output,
            "Average answer time: {:.1} seconds",
            average.as_secs_f64()
        )?,
        None => writeln!(output, "Average answer time: N/A")?,
    }

    if let Some(best) = stats.best_answer_time() {
        writeln!(output, "Fastest answer: {:.1} seconds", best.as_secs_f64())?;
    }

    writeln!(
        output,
        "Total time: {:.1} seconds",
        stats.total_answer_time().as_secs_f64()
    )?;

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

    let (detected_width, detected_height) = crossterm::terminal::size()?;

    // override with cli width and height if given
    let canvas_width = cli.width.unwrap_or(detected_width);
    let terminal_height = cli.height.unwrap_or(detected_height);

    let canvas_height = terminal_height
        .checked_sub(UI_ROWS)
        .context("terminal is too short to display the clock; increase its height")?;

    let expected = ClockTime::random();
    let show_seconds = cli.show_seconds.resolve(difficulty);

    let rendered_clock = renderer.render(expected, canvas_width, canvas_height, show_seconds);

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
    let difference_seconds = expected.analog_difference(answer);
    let correct = difficulty.accepts(expected, answer);

    writeln!(output)?;

    if correct {
        if difference_seconds == 0 {
            writeln!(output, "Correct! Exact answer.")?;
        } else {
            writeln!(
                output,
                "Correct! Your answer was off by {}.",
                format_duration(difference_seconds)
            )?;
        }
    } else {
        writeln!(
            output,
            "Incorrect. Your answer was off by {}.",
            format_duration(difference_seconds)
        )?;
    }

    writeln!(output, "The time was {}", expected.display_analog())?;
    writeln!(output, "Answered in {:.1} seconds.", elapsed.as_secs_f64())?;
    writeln!(output)?;

    Ok(RoundControl::Completed(RoundOutcome {
        difference_seconds,
        elapsed,
        correct,
    }))
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum ContinueChoice {
    Continue,
    Quit,
}

fn ask_to_continue(
    input: &mut impl BufRead,
    output: &mut impl Write,
    buffer: &mut String,
) -> anyhow::Result<ContinueChoice> {
    write!(
        output,
        "Press Enter for another clock, or q then Enter to quit: "
    )?;
    output.flush()?;

    buffer.clear();

    if input.read_line(buffer)? == 0 {
        return Ok(ContinueChoice::Quit);
    }

    Ok(if buffer.trim().eq_ignore_ascii_case("q") {
        ContinueChoice::Quit
    } else {
        ContinueChoice::Continue
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
    let minutes = total_seconds / SECONDS_PER_MINUTE;
    let seconds = total_seconds % SECONDS_PER_MINUTE;

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
        let mut input: &[u8] = b"q\n";
        let mut output = Vec::new();
        let mut buffer = String::new();

        let result = read_player_input(&mut input, &mut output, &mut buffer).unwrap();

        assert!(matches!(result, PlayerInput::Quit));
    }

    #[test]
    fn eof_quits_during_answer_prompt() {
        let mut input: &[u8] = b"";
        let mut output = Vec::new();
        let mut buffer = String::new();

        let result = read_player_input(&mut input, &mut output, &mut buffer).unwrap();

        assert!(matches!(result, PlayerInput::Quit));
    }

    #[test]
    fn invalid_input_is_retried() {
        let mut input: &[u8] = b"invalid\n9:30\n";
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
        let mut input: &[u8] = b"";
        let mut output = Vec::new();
        let mut buffer = String::new();

        let result = ask_to_continue(&mut input, &mut output, &mut buffer).unwrap();

        assert_eq!(result, ContinueChoice::Quit);
    }

    #[test]
    fn fromat_duration() {
        assert_eq!(format_duration(0), "0 seconds");
        assert_eq!(format_duration(1), "1 second");
        assert_eq!(format_duration(60), "1 minute");
        assert_eq!(format_duration(61), "1 minute and 1 second");
    }
}
