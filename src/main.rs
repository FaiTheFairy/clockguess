use std::io::{self};

use crate::{cli::Cli, render::AsciiRenderer};

use clap::Parser;

mod cli;
mod clock;
mod difficulty;
mod game;
mod render;

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    let renderer = AsciiRenderer::new(cli.theme.into());

    let stdin = io::stdin();
    let stdout = io::stdout();
    game::run(&cli, &renderer, &mut stdin.lock(), &mut stdout.lock())
}
