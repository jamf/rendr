use std::env;
use std::path::Path;

use anyhow::{anyhow, Error};
use clap::ArgMatches;

use rendr::blueprint::source;
use rendr::blueprint::Values;
use rendr::project::Project;

pub fn upgrade(args: &ArgMatches) -> Result<(), Error> {

    let working_dir = env::current_dir().map_err(|e| anyhow!("error determining working directory: {}", e))?;

    // TODO this variable is not yet used, the upgrade target must always be the latest
    let blueprint = args.value_of("blueprint").unwrap_or("original");
    let _blueprint_version = args.value_of("blueprint-version").unwrap_or("latest");

    let dir = Path::new(args.value_of("dir").unwrap_or(working_dir.to_str().unwrap()));
    let values = Values::from(args.values_of("value").unwrap());
    let dry_run = args.is_present("dry-run");

    let username = args.value_of("user");
    let env_password = env::var("GIT_PASS");
    let password = if let Ok(env_password) = &env_password {
        Some(env_password.as_str())
    } else {
        args.value_of("pass")
    };

    let provided_ssh_path = args.value_of("ssh-key").map(|p| p.as_ref());
    let callbacks = source::Source::prepare_callbacks(username, password, provided_ssh_path);

    let project = Project::new(&dir)?;

    project.upgrade(blueprint, values, Some(callbacks), dry_run).map_err(|e| anyhow!("error upgrading blueprint: {}", e))
}
