use anyhow::Error;
use clap::ArgMatches;

use rendr::blueprint::Blueprint;
use rendr::project::Project;

pub fn update(args: &ArgMatches) -> Result<(), Error> {
    // Parse CLI arguments.
    let project_path = args.value_of("project").unwrap_or(".");
    let new_blueprint = Blueprint::new(args.value_of("blueprint").unwrap(), None)?;

    // Attempt to parse the provided project.
    let project = Project::new(&project_path)?;

    project.update(&new_blueprint)?;

    Ok(())
}
