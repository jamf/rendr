use std::collections::HashMap;
use std::fs;
use std::path::Path;

use clap::{App, ArgMatches, load_yaml};
use git2::Repository;
use tempdir::TempDir;

use express::templating;

fn main() {
    let yaml = load_yaml!("cli.yml");
    let matches = App::from_yaml(yaml).get_matches();

    if let Some(matches) = matches.subcommand_matches("init") {
        init(matches);
    }
}

fn init(matches: &ArgMatches) {
    // Parse CLI arguments.
    let template = matches.value_of("template").unwrap();
    let name = matches.value_of("name").unwrap();
    let output_dir = Path::new(matches.value_of("dir").unwrap_or(name));

    let values = matches.values_of("value");
    let values: HashMap<&str, &str> = match values {
        Some(values) => values.map(parse_value).collect(),
        None         => HashMap::new(),
    };

    println!(
        "Generating project {} from template {}. Directory: {}",
        name,
        template,
        output_dir.to_str().unwrap(),
    );

    // Check out the blueprint into a new temporary directory.
    let blueprint_dir = TempDir::new("checked_out_blueprint").unwrap();

    if let Err(e) = Repository::clone(template, &blueprint_dir) {
        panic!("failed to init: {}", e);
    }

    // Create our output directory.
    fs::create_dir(output_dir).unwrap();

    // Iterate through the blueprint templates and render them into our output
    // directory.  
    if output_dir.is_dir() {
        for entry in fs::read_dir(&blueprint_dir.path().join("blueprint")).unwrap() {
            let entry = entry.unwrap();
            let path = entry.path();

            if path.is_file() {
                println!("Found file {:?}", &path);

                let filename = path.file_name().unwrap().to_str().unwrap();
                let contents = fs::read_to_string(&path).unwrap();

                let contents = templating::render_template(&contents, &values).unwrap();

                fs::write(output_dir.join(filename), &contents).unwrap();
            }
        }
    }
}

// Parse a string of "key:value" form into a tuple of (key, value).
fn parse_value(s: &str) -> (&str, &str) {
    let pos = s.find(":").unwrap();
    s.split_at(pos)
}