use anyhow::Error;

use clap::ArgMatches;
use log::info;

use rendr::project::Project;

pub fn check(args: &ArgMatches) -> Result<(), Error> {
    // Parse CLI arguments.
    let project_path = args.value_of("project").unwrap_or(".");

    // Attempt to parse the provided project.
    let project = Project::new(&project_path)?;

    project.validate()?;

    info!("Project passes validation.");

    Ok(())
}
