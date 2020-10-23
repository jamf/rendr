use std::env;
use std::error::Error;
use std::path::Path;
use std::path::PathBuf;

use clap::ArgMatches;
use log::error;

use rendr::blueprint::RendrConfig;

type DynError = Box<dyn Error>;

pub fn info(args: &ArgMatches) -> Result<(), DynError> {
    let dir: PathBuf;
    if args.is_present("dir") {
        dir = Path::new(args.value_of("dir").unwrap()).to_path_buf();
    } else {
        dir = env::current_dir().unwrap();
    }

    let path = dir.join(Path::new(".rendr.yaml"));
    if !path.exists() {
        error!("This directory does not appear to be a Rendr project: no .rendr.yaml file found");
        error!("  Expected file at {}", path.display());
        panic!("Project info not available");
    }

    let config = match RendrConfig::load(&path) {
        Ok(c) => c.unwrap(),
        Err(e) => return Err(e),
    };
    println!("{}", config);

    Ok(())
}
