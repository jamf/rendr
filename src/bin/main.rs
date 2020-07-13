mod init;

use std::result::Result;
use std::error::Error;

use clap::{App, load_yaml, crate_version};
use simplelog::{ConfigBuilder, LevelFilter, TermLogger, TerminalMode};
use log::error;

type DynError = Box<dyn Error>;

fn main() {
    let _logger = TermLogger::init(
            LevelFilter::Info,
            ConfigBuilder::new()
                .set_time_level(LevelFilter::Off)
                .build(),
            TerminalMode::Mixed)
        .expect("No interactive terminal.");

    if let Err(err) = run_app() {
        #[cfg(debug)]
        error!("{:?}", err);

        #[cfg(not(debug))]
        error!("{}", err);

        std::process::exit(1);
    }
}

fn run_app() -> Result<(), DynError> {
    let yaml = load_yaml!("cli.yml");
    let matches = App::from_yaml(yaml).version(crate_version!()).get_matches();

    if let Some(args) = matches.subcommand_matches("init") {
        init::init(args)?;
    }

    Ok(())
}
