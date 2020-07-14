mod init;

use std::result::Result;
use std::error::Error;

use clap::{App, load_yaml, crate_version};
use env_logger::{self, Env};
use log::error;

type DynError = Box<dyn Error>;

fn main() {
    init_logger();

    if let Err(err) = run_app() {
        #[cfg(debug)]
        error!("{:?}", err);

        #[cfg(not(debug))]
        error!("{}", err);

        std::process::exit(1);
    };
}

/// Initializes the logger. It'll be more verbose by default in dev builds and
/// more "tidy" in releases. It can be customized via env variables. Mostly
/// this means setting RUST_LOG to one of:
/// "OFF", "ERROR", "WARN", "INFO", "DEBUG", "TRACE"
/// 
/// More fine-grained options can be found here:
/// https://docs.rs/env_logger
fn init_logger() {
    #[cfg(debug)]
    env_logger::from_env(Env::default().default_filter_or("debug"))
        .format_timestamp(None)
        .init();

    #[cfg(not(debug))]
    env_logger::from_env(Env::default().default_filter_or("info"))
        .format_timestamp(None)
        .format_module_path(false)
        .init();
}

fn run_app() -> Result<(), DynError> {
    let yaml = load_yaml!("cli.yml");
    let matches = App::from_yaml(yaml).version(crate_version!()).get_matches();

    if let Some(args) = matches.subcommand_matches("init") {
        init::init(args)?;
    }

    Ok(())
}
