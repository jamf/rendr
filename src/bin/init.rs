use std::error::Error;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

use clap::ArgMatches;
use git2::Repository;
use tempdir::TempDir;

use express::templating;

type DynError = Box<dyn Error>;

pub fn init(matches: &ArgMatches) -> Result<(), DynError> {
    // Parse CLI arguments.
    let template = matches.value_of("template").unwrap();
    let name = matches.value_of("name").unwrap();
    let output_dir = Path::new(matches.value_of("dir").unwrap_or(name));

    let values = matches.values_of("value");

    let values: HashMap<&str, &str> = match values {
        Some(values) => values.map(parse_value).collect(),
        None         => Ok(HashMap::new()),
    }?;

    println!(
        "Generating project {} from template {}. Directory: {}",
        name,
        template,
        output_dir.to_str()
            .expect("Invalid utf8 in output_dir. This panic shouldn't happen!"),
    );

    // Check out the blueprint into a new temporary directory.
    let blueprint_dir = TempDir::new("checked_out_blueprint")?;

    Repository::clone(template, &blueprint_dir)?;

    // Create our output directory.
    fs::create_dir(output_dir)?;

    // Iterate through the blueprint templates and render them into our output
    // directory.  
    if output_dir.is_dir() {
        for entry in fs::read_dir(&blueprint_dir.path().join("blueprint"))? {
            let path = entry?.path();

            if path.is_file() {
                println!("Found file {:?}", &path);

                let filename = path.file_name()
                    .unwrap()
                    .to_str()
                    .expect("Invalid utf8 in filepath.");

                let contents = fs::read_to_string(&path)?;

                let contents = templating::render_template(&contents, &values)?;

                fs::write(output_dir.join(filename), &contents)?;
            }
        }
    }

    Ok(())
}

// Parse a string of "key:value" form into a tuple of (key, value).
fn parse_value(s: &str) -> Result<(&str, &str), String> {
    let pos = s.find(":")
        .ok_or(format!("Invalid value `{}`", s))?;

    Ok(s.split_at(pos))
}