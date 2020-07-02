mod init;

use std::result::Result;
use std::error::Error;

use clap::{App, load_yaml, crate_version};

type DynError = Box<dyn Error>;

fn main() {
    if let Err(err) = run_app() {
        #[cfg(debug)]
        eprintln!("Error: {:?}", err);

        #[cfg(not(debug))]
        eprintln!("Error: {}", err);

        std::process::exit(1);
    }
}

fn run_app() -> Result<(), DynError> {
    let yaml = load_yaml!("cli.yml");
    let matches = App::from_yaml(yaml).version(crate_version!()).get_matches();

    if let Some(matches) = matches.subcommand_matches("init") {
        init::init(matches)?;
    }

    Ok(())
}
