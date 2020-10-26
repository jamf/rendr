use std::env;

use anyhow::{anyhow, Error};
use clap::ArgMatches;

use rendr::project::Project;

pub fn update(args: &ArgMatches) -> Result<(), Error> {

    let working_dir = env::current_dir().map_err(|e| anyhow!("error determining working directory: {}", e))?;

    // Parse CLI arguments.
    let project_path = args.value_of("dir").unwrap_or(working_dir.to_str().unwrap());

    // Attempt to parse the provided project.
    let project = Project::new(&project_path)?;

    project.upgrade_blueprint_with_templates()?;

    Ok(())
}
