use std::error::Error;

use clap::ArgMatches;

type DynError = Box<dyn Error>;

pub fn upgrade(args: &ArgMatches) -> Result<(), DynError> {
    let blueprint_version = args.value_of("blueprint-version").unwrap_or("latest");

    Ok(())
}
