use std::collections::HashMap;
use std::error::Error;
use std::io::{self, Write};
use std::path::Path;
use std::sync::mpsc::channel;
use std::time::Duration;

use clap::ArgMatches;
use git2::{Oid, Repository, IndexAddOption, Signature};
use log::{info, debug, error};
use text_io::read;
use notify::{Watcher, RecursiveMode, watcher};

use rendr::templating;
use rendr::blueprint::{Blueprint, ValueSpec};

type DynError = Box<dyn Error>;

pub fn init(args: &ArgMatches) -> Result<(), DynError> {
    let blueprint_path = args.value_of("blueprint").unwrap();
    let name = args.value_of("name").unwrap();
    let scaffold_path = Path::new(args.value_of("dir").unwrap_or(name));

    let blueprint = Blueprint::new(blueprint_path)?;

    // Time to parse values. Let's start by collecting the defaults.
    let mut values: HashMap<&str, &str> = blueprint.default_values()
                                                   .collect();

    // If some values were provided via CLI arguments, merge those in.
    if let Some(cli_values) = args.values_of("value") {
        let cli_values: Result<Vec<_>, _> = cli_values.map(parse_value).collect();
        values.extend(cli_values?);
    }

    // Figure out which required values are still missing.
    let missing_values = blueprint.required_values()
        .filter(|v| values.get::<str>(&v.name).is_none());

    // Prompt for the missing values and collect them.
    let prompt_values_owned: Vec<_> = prompt_for_values(missing_values).collect();

    // Merge the values from prompts in.
    let prompt_values: Vec<_> = prompt_values_owned.iter()
        .map(|(k, v)| (*k, v.as_str()))
        .collect();
    values.extend(prompt_values);

    init_scaffold(args, &values)?;

    if args.is_present("watch") {
        info!("Watching for blueprint changes...");

        // Create a channel to receive the events.
        let (tx, rx) = channel();

        // Create a watcher object, delivering debounced events.
        // The notification back-end is selected based on the platform.
        let mut watcher = watcher(tx, Duration::from_secs(1))?;

        // Add a path to be watched. All files and directories at that path and
        // below will be monitored for changes.
        watcher.watch(blueprint_path, RecursiveMode::Recursive)?;

        loop {
            match rx.recv() {
                Ok(event) => {
                    debug!("Watch event: {:?}", event);
                    info!("Blueprint changed! Recreating scaffold...");
                    std::fs::remove_dir_all(scaffold_path)?;
                    init_scaffold(args, &values)?;
                    info!("Success!");
                },
                Err(e) => error!("watch error: {:?}", e),
            }
        }
    }

    Ok(())
}

fn init_scaffold(args: &ArgMatches, values: &HashMap<&str, &str>) -> Result<(), DynError> {
    // Parse CLI arguments.
    let blueprint_path = args.value_of("blueprint").unwrap();
    let name = args.value_of("name").unwrap();
    let output_dir = Path::new(args.value_of("dir").unwrap_or(name));

    // Attempt to read the provided blueprint.
    let blueprint = Blueprint::new(blueprint_path)?;

    println!("{}", blueprint);

    info!("Output directory: {:?}. Creating your new scaffold...", &output_dir);

    let mustache = templating::Mustache::new();

    blueprint.render(&mustache, &values, &output_dir)?;

    if args.is_present("git-init") || (blueprint.is_git_init_enabled() && !args.is_present("no-git-init")) {
        debug!("Initializing Git repository");
        git_init(&output_dir)?;
    }

    info!("Success. Enjoy!");

    Ok(())
}

fn git_init(dir: &Path) -> Result<Oid, git2::Error> {
    let repo = Repository::init(dir).expect("failed to initialize Git repository");

    // First use the config to initialize a commit signature for the user
    let sig = match repo.signature() {
        Ok(signature) => signature,
        Err(_) => Signature::now("rendr", "rendr@github.com")?,
    };

    let tree_id = {
        let mut index = repo.index()?;
        index.add_all(["."].iter(), IndexAddOption::DEFAULT, None)?;
        index.write()?;
        index.write_tree()?
    };

    let message = "Initial project generated with rendr";
    let tree = repo.find_tree(tree_id)?;
    repo.commit(Some("HEAD"), &sig, &sig, message, &tree, &[])
}

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

#[test]
fn correct_values_are_parsed_correctly() {
    let (foo, bar) = parse_value("foo:bar").unwrap();

    assert_eq!(foo, "foo");
    assert_eq!(bar, "bar");
}

#[test]
fn git_init_works() -> Result<(), Box<dyn Error>>{
    use tempdir::TempDir;
    use git2::RepositoryState;

    let dir = TempDir::new("my-project").unwrap();
    std::fs::write(dir.path().join("foo"), "foo")?;

    git_init(dir.path())?;

    let repo = Repository::open(dir.path())?;

    // General "Is the repo in the expected and sane state?" tests
    assert!(!repo.is_empty()?, "the repository was empty");
    assert_eq!(RepositoryState::Clean, repo.state(), "the repository wasn't in a clean state");
    assert!(!repo.head_detached()?, "the repository head was detached");
    repo.head()?;
    assert!(dir.path().join("foo").exists());

    assert!(dir.path().join(".git/index").exists(), "the git index file doesn't exist");

    Ok(())
}
