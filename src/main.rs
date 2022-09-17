use clap::{Parser, Subcommand};
use commands::r#move::*;
use eyre::Result;

pub mod commands;
pub mod models;
pub mod utils;

#[derive(Debug, Subcommand)]
enum Command {
    /// Move one or multiple files or folders from one destination to another
    /// - Creates missing destination folders if necessary
    /// - If multiple files are given, tries to avoid conflicts
    Move(MoveArgs),
}

#[derive(Debug, Parser)]
#[clap(author, version)]
struct Cli {
    #[clap(subcommand)]
    command: Command,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Command::Move(args) => r#move(args),
    }
}
