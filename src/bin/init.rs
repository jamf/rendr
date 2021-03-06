use std::collections::HashMap;
use std::error::Error;
use std::fs;
use std::io::{self, Write};
use std::path::Path;
use std::sync::mpsc::channel;
use std::time::Duration;

use clap::ArgMatches;
use log::{debug, error, info};
use notify::DebouncedEvent;
use notify::{watcher, RecursiveMode, Watcher};
use text_io::read;

use rendr::blueprint::{Blueprint, BlueprintAuth, ValueSpec};
use rendr::templating;

type DynError = Box<dyn Error>;

pub fn init(args: &ArgMatches) -> Result<(), DynError> {
    let blueprint_path = args.value_of("blueprint").unwrap();
    let scaffold_path = Path::new(args.value_of("dir").unwrap_or("."));

    let username = args.value_of("user").map(|s| s.to_string());
    let password = args.value_of("password").map(|s| s.to_string());
    let ssh_key = args.value_of("ssh-key").map(|s| s.to_string());
    let auth = BlueprintAuth::new(username, password, ssh_key);

    let blueprint = Blueprint::new(blueprint_path, Some(auth))?;

    // Time to parse values. Let's start by collecting the defaults.
    let mut values: HashMap<&str, &str> = blueprint.default_values().collect();

    // If some values were provided via CLI arguments, merge those in.
    if let Some(cli_values) = args.values_of("value") {
        let cli_values: Result<Vec<_>, _> = cli_values.map(parse_value).collect();
        values.extend(cli_values?);
    }

    // Figure out which required values are still missing.
    let missing_values = blueprint
        .required_values()
        .filter(|v| values.get::<str>(&v.name).is_none());

    // Prompt for the missing values and collect them.
    let prompt_values_owned: Vec<_> = prompt_for_values(missing_values).collect();

    // Merge the values from prompts in.
    let prompt_values: Vec<_> = prompt_values_owned
        .iter()
        .map(|(k, v)| (*k, v.as_str()))
        .collect();
    values.extend(prompt_values);

    init_scaffold(&blueprint, args, &values)?;

    if args.is_present("watch") {
        watch(&blueprint, scaffold_path, args, &values)?;
    }

    Ok(())
}

fn init_scaffold(
    blueprint: &Blueprint,
    args: &ArgMatches,
    values: &HashMap<&str, &str>,
) -> Result<(), DynError> {
    // Parse CLI arguments.
    let output_dir = Path::new(args.value_of("dir").unwrap_or("."));
    let dry_run = args.is_present("dry-run");

    println!("{}", blueprint);

    info!(
        "Output directory: {:?}. Creating your new scaffold...",
        &output_dir
    );

    let engine = templating::Tmplpp::new();
    blueprint.render(
        &engine,
        &values.into(),
        &output_dir,
        args.is_present("git-init"),
        args.is_present("no-git-init"),
        dry_run,
    )?;
    info!("Success. Enjoy!");

    Ok(())
}

fn watch(
    blueprint: &Blueprint,
    scaffold_path: impl AsRef<Path> + Copy,
    args: &ArgMatches,
    values: &HashMap<&str, &str>,
) -> Result<(), DynError> {
    info!("Watching for blueprint changes...");

    // Create a channel to receive the events.
    let (tx, rx) = channel();

    // Create a watcher object, delivering debounced events.
    // The notification back-end is selected based on the platform.
    let mut watcher = watcher(tx, Duration::from_secs(1))?;

    // Add a path to be watched. All files and directories at that path and
    // below will be monitored for changes.
    watcher.watch(blueprint.path(), RecursiveMode::Recursive)?;

    loop {
        match rx.recv() {
            Ok(event) => {
                debug!("Watch event: {:?}", event);
                match event {
                    #[allow(unused_variables)]
                    DebouncedEvent::Create(path)
                    | DebouncedEvent::Chmod(path)
                    | DebouncedEvent::Remove(path)
                    | DebouncedEvent::Rename(path, _)
                    | DebouncedEvent::Write(path) => {
                        info!("");
                        info!("Blueprint changed! Recreating scaffold...");
                        rm_all(scaffold_path.as_ref())?;
                        if let Err(e) = init_scaffold(blueprint, args, values) {
                            error!("{}", e);
                        }
                    }
                    _ => debug!("Skipping event {:?}", event),
                }
            }
            Err(e) => error!("watch error: {:?}", e),
        }
    }
}

fn rm_all(p: &Path) -> Result<(), DynError> {
    let paths = fs::read_dir(p).unwrap();

    for path in paths {
        let path = path.unwrap().path();
        if path.is_dir() {
            fs::remove_dir_all(path)?;
        } else {
            fs::remove_file(path)?;
        }
    }

    Ok(())
}

type ValueFromPrompt<'s> = (&'s str, String);

fn prompt_for_values<'s>(
    values: impl Iterator<Item = &'s ValueSpec>,
) -> impl Iterator<Item = ValueFromPrompt<'s>> {
    values.map(prompt_for_value)
}

fn prompt_for_value(value: &ValueSpec) -> ValueFromPrompt<'_> {
    print!("{}: ", value.description);
    io::stdout().flush().unwrap();
    (&value.name, read!("{}\n"))
}

fn parse_value(s: &str) -> Result<(&str, &str), String> {
    let pos = s.find(":").ok_or(format!("Invalid value `{}`", s))?;

    let mut result = s.split_at(pos);
    result.1 = &result.1[1..];

    Ok((result.0, result.1))
}

#[test]
fn correct_values_are_parsed_correctly() {
    let (foo, bar) = parse_value("foo:bar").unwrap();

    assert_eq!(foo, "foo");
    assert_eq!(bar, "bar");
}
