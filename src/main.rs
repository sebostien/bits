use anyhow::{anyhow, Result};
use clap::{arg, Command, CommandFactory, Parser, Subcommand};
use clap_complete::{generate, Generator, Shell};
use std::{io, path::PathBuf, process::ExitCode};

mod config;
mod git;
mod open;

use config::Config;

#[derive(Parser, Debug)]
#[command(name = "bits")]
#[command(about = "A collection of utilities", long_about = None)]
pub struct Cli {
    /// Path to the config file.
    /// [default: `$XDG_CONFIG_HOME/bits/config.toml`]
    #[arg(short, long)]
    pub config_file: Option<PathBuf>,
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    Open { text: String },
    Completions { shell: Shell },
}

fn print_completions<G: Generator>(generator: G, cmd: &mut Command) {
    generate(
        generator,
        cmd,
        cmd.get_name().to_string(),
        &mut io::stdout(),
    );
}

fn main() -> ExitCode {
    let args = Cli::parse();
    if let Err(e) = run(args) {
        eprintln!("{e}");
        ExitCode::FAILURE
    } else {
        ExitCode::SUCCESS
    }
}

pub fn run(args: Cli) -> Result<()> {
    let config_file = if let Some(c) = args.config_file {
        c
    } else if let Some(mut home) = dirs::config_dir() {
        home.push("bits/config.toml");
        home
    } else {
        return Err(anyhow!(
            "Could not detect your home directory. Please use the `--config-file` option"
        ));
    };

    let config = Config::from_file(config_file)?;

    match args.command {
        Commands::Open { text } => config.open.open(&text),
        Commands::Completions { shell } => {
            print_completions(shell, &mut Cli::command());
            Ok(())
        }
    }
}
