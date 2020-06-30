use std::error::Error;
use std::collections::HashMap;
use std::path::Path;
use std::io;
use std::io::prelude::*;

use clap::ArgMatches;
use serde::Deserialize;
use text_io::read;

use express::templating;
use express::blueprint::Blueprint;
use express::blueprint::ValueSpec;
use express::utilities::parse_value;

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

    let blueprint = Blueprint::new(template)?;

    println!("{}", blueprint);
    println!(
        "Output directory: {}",
        output_dir.to_str()
            .expect("Invalid utf8 in output_dir. This panic shouldn't happen!"),
    );

    // TODO: prompt for values not provided

    let mustache = templating::Mustache::new();

    blueprint.render(&mustache, &values, &output_dir)?;

    Ok(())
}

fn prompt_for_values(values: &HashMap<&str, &str>, blueprint: &Blueprint) {
    for value in blueprint.values() {
        match values.get::<str>(&value.name) {
            Some(_) => (),
            None => prompt_for_value(values, value)
        }
    }
}

fn prompt_for_value(values: &HashMap<&str, &str>, value: &ValueSpec) {
    println!("[{}] {}: ", value.name, value.description);
    let line: String = read!("{}\n");
    let key = value.name;
    values.insert(key, &line); // TODO: This isn't working
}

#[test]
fn correct_values_are_parsed_correctly() {
    let (foo, bar) = parse_value("foo:bar").unwrap();

    assert_eq!(foo, "foo");
    assert_eq!(bar, "bar");
}