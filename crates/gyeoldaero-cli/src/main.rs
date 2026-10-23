//! The `gyeoldaero` binary: argument parsing and subcommand wiring.

use std::process::ExitCode;

use clap::{Parser, Subcommand};

/// find the grain of what you're learning — a terminal notebook where every note is a folder.
#[derive(Debug, Parser)]
#[command(name = "gyeoldaero", version, about)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Write the static site for a notebook.
    Build,
    /// Build, watch and serve a notebook.
    Serve,
    /// Report problems in a notebook.
    Check,
    /// Create a note (folder + index.md).
    New,
}

impl Command {
    /// The PRD that implements this subcommand.
    fn prd(&self) -> &'static str {
        match self {
            Command::Build => "PRD-002",
            Command::Serve => "PRD-005",
            Command::Check => "PRD-001",
            Command::New => "PRD-006",
        }
    }
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    eprintln!("not implemented yet ({})", cli.command.prd());
    ExitCode::from(2)
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::CommandFactory;

    #[test]
    fn cli_definition_is_valid() {
        Cli::command().debug_assert();
    }
}
