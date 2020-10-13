use std::collections::HashMap;
use std::env;
use std::error::Error;
use std::io::{self, Write};
use std::path::Path;
use std::sync::mpsc::channel;
use std::time::Duration;

use clap::ArgMatches;
use git2::{Cred, IndexAddOption, Oid, RemoteCallbacks, Repository, Signature};
use log::{debug, error, info};
use notify::{watcher, RecursiveMode, Watcher};
use text_io::read;

use rendr::blueprint::{Blueprint, ValueSpec};
use rendr::templating;

type DynError = Box<dyn Error>;

pub fn init(args: &ArgMatches) -> Result<(), DynError> {
    let blueprint_path = args.value_of("blueprint").unwrap();
    let name = args.value_of("name").unwrap();
    let scaffold_path = Path::new(args.value_of("dir").unwrap_or(name));

    let username = args.value_of("user");
    let env_password = env::var("GIT_PASS");
    let password = if let Ok(env_password) = &env_password {
        Some(env_password.as_str())
    } else {
        args.value_of("pass")
    };

    let provided_ssh_path = args.value_of("ssh-key").map(|p| p.as_ref());

    let callbacks = prepare_callbacks(username, password, provided_ssh_path);
    let blueprint = Blueprint::new(blueprint_path, Some(callbacks))?;

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
                info!("Blueprint changed! Recreating scaffold...");
                std::fs::remove_dir_all(scaffold_path)?;
                if let Err(e) = init_scaffold(blueprint, args, values) {
                    error!("{}", e);
                }
            }
            Err(e) => error!("watch error: {:?}", e),
        }
    }
}

fn init_scaffold(
    blueprint: &Blueprint,
    args: &ArgMatches,
    values: &HashMap<&str, &str>,
) -> Result<(), DynError> {
    // Parse CLI arguments.
    let name = args.value_of("name").unwrap();
    let output_dir = Path::new(args.value_of("dir").unwrap_or(name));

    println!("{}", blueprint);

    info!(
        "Output directory: {:?}. Creating your new scaffold...",
        &output_dir
    );

    let mustache = templating::Mustache::new();

    blueprint.render(&mustache, &values, &output_dir)?;

    if args.is_present("git-init")
        || (blueprint.is_git_init_enabled() && !args.is_present("no-git-init"))
    {
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

fn prepare_callbacks<'c>(
    provided_user: Option<&'c str>,
    provided_pass: Option<&'c str>,
    provided_ssh_key: Option<&'c Path>,
) -> RemoteCallbacks<'c> {
    let mut callbacks = RemoteCallbacks::new();
    let mut auth_retries = 3;

    callbacks.credentials(move |_url, username_from_url, allowed_types| {
        debug!("Git requested cred types: {:?}", allowed_types);

        if auth_retries < 1 {
            panic!("exceeded 3 auth retries; invalid credentials?");
        }

        if allowed_types.is_ssh_key() {
            auth_retries -= 1;

            if let Some(path) = provided_ssh_key {
                return Cred::ssh_key(
                    &get_username(provided_user, username_from_url).unwrap(),
                    None,
                    path,
                    None,
                );
            } else {
                return Cred::ssh_key(
                    &get_username(provided_user, username_from_url).unwrap(),
                    None,
                    &Path::new(&format!("{}/.ssh/id_rsa", std::env::var("HOME").unwrap())),
                    None,
                );
            }
        } else if allowed_types.is_username() {
            return Cred::username(&get_username(provided_user, username_from_url).unwrap());
        } else if allowed_types.is_user_pass_plaintext() {
            auth_retries -= 1;

            return Cred::userpass_plaintext(
                &get_username(provided_user, username_from_url).unwrap(),
                &get_password(provided_pass).unwrap(),
            );
        }

        panic!(
            "git requested an unimplemented credential type: {:?}",
            allowed_types
        )
    });

    fn get_username(
        provided_user: Option<&str>,
        username_from_url: Option<&str>,
    ) -> Result<String, DynError> {
        if let Some(username) = provided_user {
            return Ok(username.to_string());
        }

        if let Some(username) = username_from_url {
            return Ok(username.to_string());
        }

        print!("Username: ");
        io::stdout().flush().unwrap();
        Ok(read!("{}\n"))
    }

    fn get_password(provided_pass: Option<&str>) -> Result<String, DynError> {
        if let Some(pass) = provided_pass {
            return Ok(pass.to_string());
        }

        Ok(rpassword::read_password_from_tty(Some("Password: "))?)
    }

    callbacks
}

#[test]
fn correct_values_are_parsed_correctly() {
    let (foo, bar) = parse_value("foo:bar").unwrap();

    assert_eq!(foo, "foo");
    assert_eq!(bar, "bar");
}

#[test]
fn git_init_works() -> Result<(), Box<dyn Error>> {
    use git2::RepositoryState;
    use tempdir::TempDir;

    let dir = TempDir::new("my-project").unwrap();
    std::fs::write(dir.path().join("foo"), "foo")?;

    git_init(dir.path())?;

    let repo = Repository::open(dir.path())?;

    // General "Is the repo in the expected and sane state?" tests
    assert!(!repo.is_empty()?, "the repository was empty");
    assert_eq!(
        RepositoryState::Clean,
        repo.state(),
        "the repository wasn't in a clean state"
    );
    assert!(!repo.head_detached()?, "the repository head was detached");
    repo.head()?;
    assert!(dir.path().join("foo").exists());

    assert!(
        dir.path().join(".git/index").exists(),
        "the git index file doesn't exist"
    );

    Ok(())
}
