use std::collections::HashMap;
use std::env;
use std::error::Error;
use std::io::{self, Write};
use std::path::Path;
use std::path::PathBuf;

use clap::ArgMatches;
use log::{info, debug, error};
use text_io::read;

use rendr::templating;
use rendr::blueprint::Blueprint;
use rendr::blueprint::RendrConfig;
use rendr::blueprint::ValueSpec;

type DynError = Box<dyn Error>;

pub fn upgrade(args: &ArgMatches) -> Result<(), DynError> {
    // TODO this variable is not yet used, the upgrade target must always be the latest
    let dry_run = args.is_present("dry-run");
    debug!("Upgrade dry run mode: {}", dry_run);
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

    // Initialize values with blueprint defaults
    let mut values: HashMap<&str, &str> = blueprint.default_values().collect();

    // Add values from original project generation
    let config_values: HashMap<&str, &str> = config.values.iter()
        .map(|v| (v.name.as_str(), v.value.as_str()))
        .collect();
    values.extend(config_values);

    // If some values were provided via CLI arguments, merge those in
    if let Some(cli_values) = args.values_of("value") {
        let cli_values: Result<Vec<_>, _> = cli_values.map(parse_value).collect();
        values.extend(cli_values?);
    }

    // Figure out which required values are still missing
    let missing_values = blueprint.required_values()
        .filter(|v| values.get::<str>(&v.name).is_none());

    // Prompt for the missing values and collect them
    let prompt_values_owned: Vec<_> = prompt_for_values(missing_values).collect();

    // Merge the values from prompts in
    let prompt_values: Vec<_> = prompt_values_owned.iter()
        .map(|(k, v)| (*k, v.as_str()))
        .collect();
    values.extend(prompt_values);

    // Update the target version, inserting if it does not exist for some reason
    let target_version = blueprint.metadata.version.to_string();
    let v = values.entry("version").or_insert(target_version.as_str());
    *v = target_version.as_str();

    debug!("Rendering blueprint with values:");
    for (k, v) in values.clone() {
        debug!("- {}: {}", k, v);
    }

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

// TODO move this code to a common spot, copied from init.rs
type ValueFromPrompt<'s> = (&'s str, String);

fn prompt_for_values<'s>(values: impl Iterator<Item = &'s ValueSpec>) -> impl Iterator<Item = ValueFromPrompt<'s>> {
    values.map(prompt_for_value)
}

fn prompt_for_value(value: &ValueSpec) -> ValueFromPrompt<'_> {
    print!("{}: ", value.description);
    io::stdout().flush().unwrap();
    (&value.name, read!("{}\n"))
}

fn parse_value(s: &str) -> Result<(&str, &str), String> {
    let pos = s.find(":")
        .ok_or(format!("Invalid value `{}`", s))?;

    let mut result = s.split_at(pos);
    result.1 = &result.1[1..];

    Ok((result.0, result.1))
}

