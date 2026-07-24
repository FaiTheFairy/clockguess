use std::io;

use crate::{cli::Cli, render::AsciiRenderer};

use clap::Parser;

mod cli;
mod clock;
mod difficulty;
mod game;
mod render;
mod score;

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    let renderer = AsciiRenderer::new(cli.theme.into());

    let stdin = io::stdin();
    let stdout = io::stdout();

    let mut input = stdin.lock();
    let mut output = stdout.lock();

    game::run(&cli, &renderer, &mut input, &mut output)
}
