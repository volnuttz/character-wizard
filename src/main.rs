//! Native character-wizard executable entry point.

use std::process::ExitCode;

fn main() -> ExitCode {
    character_wizard_cli::cli::main()
}
