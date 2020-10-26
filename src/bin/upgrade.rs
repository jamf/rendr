use std::env;
use std::path::Path;

use anyhow::{anyhow, Error};
use clap::ArgMatches;

use rendr::blueprint::Values;
use rendr::project::Project;

pub fn upgrade(args: &ArgMatches) -> Result<(), Error> {

    let working_dir = env::current_dir().map_err(|e| anyhow!("error determining working directory: {}", e))?;

    // TODO this variable is not yet used, the upgrade target must always be the latest
    let _blueprint_version = args.value_of("blueprint-version").unwrap_or("latest");

    let dir = Path::new(args.value_of("dir").unwrap_or(working_dir.to_str().unwrap()));
    let values = Values::from(args.values_of("value").unwrap());
    let dry_run = args.is_present("dry-run");

    let project = Project::new(&dir)?;

    project.upgrade_blueprint_with_scripts(values, dry_run).map_err(|e| anyhow!("error upgrading blueprint: {}", e))
}
