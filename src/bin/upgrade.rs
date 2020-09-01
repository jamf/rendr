use std::collections::HashMap;
use std::env;
use std::error::Error;
use std::path::Path;
use std::path::PathBuf;

use clap::ArgMatches;
use log::{info, debug, error};

use rendr::templating;
use rendr::blueprint::Blueprint;
use rendr::blueprint::RendrConfig;

type DynError = Box<dyn Error>;

pub fn upgrade(args: &ArgMatches) -> Result<(), DynError> {
    let blueprint_version = args.value_of("blueprint-version").unwrap_or("latest");
    let working_dir = match env::current_dir() {
        Ok(dir) => dir,
        Err(e) => return Err(Box::new(e)),
    };
    let dir = Path::new(args.value_of("dir").unwrap_or(working_dir.to_str().unwrap()));

    let config = match load_rendr_config(&dir) {
        Ok(c) => c,
        Err(e) => return Err(e),
    };

    // Attempt to read the provided blueprint
    let relative_source = dir.join(&config.source);
    let blueprint = match relative_source.exists() {
        true =>  Blueprint::new(relative_source.to_str().unwrap())?,
        false => Blueprint::new(config.source.as_str())?,
    };

    println!("{}", blueprint);

    // Check if blueprint version can be updated
    if blueprint.metadata.version == config.version {
        println!("Project is already on the latest blueprint version (v{})", config.version);
        return Ok(())
    } else if blueprint.metadata.version < config.version {
        println!("Project is on a newer version of the blueprint. Something might be wrong.");
        println!("  Project version:   {}", config.version);
        println!("  Blueprint version: {}", blueprint.metadata.version);
        panic!("Canceling upgrade");
    }

    println!("Upgrading project from blueprint version {}", blueprint.metadata.version);

    // Parse values, same as init
    let mut values: HashMap<&str, &str> = blueprint.default_values()
                                                   .collect();

    // Prompt for missing values

    // Render new templates
    let mustache = templating::Mustache::new();
    blueprint.render_upgrade(&mustache, &values, &dir, &config.source);

    Ok(())
}

fn load_rendr_config(dir: &Path) -> Result<RendrConfig, DynError> {
    let path = dir.join(Path::new(".rendr.yaml"));
    if !path.exists() {
        error!("This directory does not appear to be a Rendr project: no .rendr.yaml file found");
        error!("  Expected file at {}", path.display());
        panic!("Project info not available");
    }

    match RendrConfig::load(&path) {
        Ok(c) => Ok(c.unwrap()),
        Err(e) => Err(e),
    }
}
