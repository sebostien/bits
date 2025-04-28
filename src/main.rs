use anyhow::Result;
use clap::{arg, Command, CommandFactory, Parser, Subcommand};
use clap_complete::{generate, Generator, Shell};
use log::error;
use std::{io, path::PathBuf, process::ExitCode};
use term_colors::TermColors;

mod config;
mod git;
mod open;
mod term_colors;

use config::Config;

#[derive(Parser, Debug)]
#[command(name = "bits")]
#[command(about = "A collection of utilities", long_about = None)]
pub struct Cli {
    /// Path to the config file
    /// [default: `$XDG_CONFIG_HOME/bits/config.toml`]
    #[arg(short, long)]
    config_file: Option<PathBuf>,
    /// Increase logging verbosity
    #[arg(short, action=clap::ArgAction::Count, global=true)]
    verbosity: u8,
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    Open { text: String },
    PrintColors,
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
    init_log(args.verbosity);

    if let Err(e) = run(args) {
        error!("{e}");
        ExitCode::FAILURE
    } else {
        ExitCode::SUCCESS
    }
}

pub fn run(args: Cli) -> Result<()> {
    let config = Config::new(args.config_file)?;

    match args.command {
        Commands::Open { text } => config.open.open(&text),
        Commands::PrintColors => TermColors::print_colors(),
        Commands::Completions { shell } => {
            print_completions(shell, &mut Cli::command());
            Ok(())
        }
    }
}

fn init_log(verbosity: u8) {
    let mut logger = env_logger::builder();
    logger.parse_default_env().format_timestamp_secs();

    match verbosity {
        1 => {
            logger.filter_level(log::LevelFilter::Warn);
        }
        2 => {
            logger.filter_level(log::LevelFilter::Info);
        }
        3 => {
            logger.filter_level(log::LevelFilter::Debug);
        }
        4.. => {
            logger.filter_level(log::LevelFilter::Trace);
        }
        _ => {}
    };

    logger.init();
}
