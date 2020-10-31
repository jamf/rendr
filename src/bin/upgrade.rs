use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Error};
use clap::ArgMatches;
use log::{debug, error};

use rendr::blueprint::Blueprint;
use rendr::blueprint::BlueprintAuth;
use rendr::blueprint::RendrConfig;
use rendr::blueprint::Values;
use rendr::project::Project;

pub fn upgrade(args: &ArgMatches) -> Result<(), Error> {

    let working_dir: PathBuf = env::current_dir().map_err(|e| anyhow!("error determining working directory: {}", e))?;
    let dir = Path::new(args.value_of("dir").unwrap_or(working_dir.to_str().unwrap()));
    let blueprint_source = args.value_of("blueprint");
    let values = match args.is_present("value") {
        true => Values::from(args.values_of("value").unwrap()),
        false => Values::new(),
    };
    let dry_run = args.is_present("dry-run");

    let username = args.value_of("user").map(|s| s.to_string());
    let password = args.value_of("password").map(|s| s.to_string());
    let ssh_key = args.value_of("ssh-key").map(|s| s.to_string());
    let auth = BlueprintAuth::new(username, password, ssh_key);

    let rendr_file: PathBuf = dir.join(Path::new(".rendr.yaml"));
    if !rendr_file.exists() {
        error!("This directory does not appear to be a Rendr project: no .rendr.yaml file found");
        error!("  Expected file at {}", rendr_file.display());
        panic!("Project metadata not available");
    }

    let yaml = fs::read_to_string(rendr_file)?;
    let config: RendrConfig = serde_yaml::from_str(&yaml)?;

    let relative_source = dir.join(config.source.clone());

    debug!("Locating blueprint source, checking if relative source exists: {}", relative_source.display());
    // let blueprint = Blueprint::new(config.source.as_str(), Some(auth));
    let blueprint = match relative_source.exists() {
        true => Blueprint::new(relative_source.as_os_str().to_str().unwrap(), Some(auth)),
        false => Blueprint::new(config.source.as_str(), Some(auth)),
    };

    let mut project = Project::new(&dir, blueprint.unwrap())?;

    project.upgrade(blueprint_source, values, dry_run).map_err(|e| anyhow!("error upgrading blueprint: {}", e))
}