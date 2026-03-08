use std::process::ExitCode;

use clap::Parser;
use todox::cli::Cli;

fn main() -> ExitCode {
    let cli = Cli::parse();

    match todox::run(cli.clone()) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("{}", todox::localized_error(&cli, &error));
            ExitCode::FAILURE
        }
    }
}
