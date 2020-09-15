use std::error::Error;

use clap::ArgMatches;

use rendr::project::Project;

type DynError = Box<dyn Error>;

pub fn check(args: &ArgMatches) -> Result<(), DynError> {
    // Parse CLI arguments.
    let project_path = args.value_of("project").unwrap_or(".");

    // Attempt to parse the provided project.
    let project = Project::new(&project_path)?;

    project.validate()?;

    Ok(())
}
