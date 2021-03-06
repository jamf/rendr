mod check;
mod create_blueprint;
mod info;
mod init;
mod upgrade;

use std::env;
use std::error::Error;
use std::result::Result;

use clap::{crate_version, load_yaml, App};
use env_logger::{self, Env};
use log::error;

type DynError = Box<dyn Error>;

fn main() {
    let args: Vec<String> = env::args().collect();
    let dry_run = args.iter().any(|f| f == "--dry-run");
    let debug = args.iter().any(|f| f == "--debug");
    let log_level_from_args = match debug || dry_run {
        true => "debug",
        false => "info",
    };
    init_logger(log_level_from_args);

    if let Err(err) = run_app() {
        #[cfg(debug)]
        error!("{:?}", err);

        #[cfg(not(debug))]
        error!("{}", err);

        std::process::exit(1);
    };
}

const LOG_LEVEL_ENV_VAR: &str = "RENDR_LOG";

/// Initializes the logger. It'll be more verbose by default in dev builds and
/// more "tidy" in releases. It can be customized via env variables. Mostly
/// this means setting RENDR_LOG to one of:
/// "OFF", "ERROR", "WARN", "INFO", "DEBUG", "TRACE"
///
/// More fine-grained options can be found here:
/// https://docs.rs/env_logger
fn init_logger(_log_level_from_args: &str) {
    #[cfg(debug)]
    env_logger::from_env(Env::default().filter_or(LOG_LEVEL_ENV_VAR, "debug"))
        .format_timestamp(None)
        .init();

    #[cfg(not(debug))]
    {
        let mut logger_builder =
            env_logger::from_env(Env::default().filter_or(LOG_LEVEL_ENV_VAR, _log_level_from_args));

        // Turn off the prefix completely unless the logging level env var
        // is explicitly specified.
        if let Err(_) = env::var(LOG_LEVEL_ENV_VAR) {
            logger_builder.format_level(false);
        }

        logger_builder
            .format_timestamp(None)
            .format_module_path(false)
            .init();
    }
}

fn run_app() -> Result<(), DynError> {
    let yaml = load_yaml!("cli.yml");
    let matches = App::from_yaml(yaml).version(crate_version!()).get_matches();

    match matches.subcommand() {
        ("create", Some(args)) => init::init(args)?,
        ("create-blueprint", Some(args)) => create_blueprint::create(args)?,
        ("check", Some(args)) => check::check(args)?,
        ("info", Some(args)) => info::info(args)?,
        ("upgrade", Some(args)) => upgrade::upgrade(args)?,
        _ => panic!("unknown subcommand"),
    }

    Ok(())
}
