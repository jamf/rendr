use std::env;
use std::error::Error;
use std::path::Path;

use clap::ArgMatches;

use rendr::blueprint::Values;
use rendr::project::Project;

type DynError = Box<dyn Error>;


pub fn upgrade(args: &ArgMatches) -> Result<(), DynError> {

    let working_dir = match env::current_dir() {
        Ok(dir) => dir,
        Err(e) => return Err(Box::new(e)),
    };

    // TODO this variable is not yet used, the upgrade target must always be the latest
    let _blueprint_version = args.value_of("blueprint-version").unwrap_or("latest");

    let dry_run = args.is_present("dry-run");
    let dir = Path::new(args.value_of("dir").unwrap_or(working_dir.to_str().unwrap()));
    let values = Values::from(args.values_of("value").unwrap());

    let project = Project::new(&dir)?;

    project.upgrade_blueprint_with_scripts(values, dry_run)?;

    Ok(())
}
