use std::error::Error;
use std::collections::HashMap;
use std::path::Path;
use std::io::{self, Write};

use clap::ArgMatches;
use text_io::read;

use express::templating;
use express::blueprint::Blueprint;
use express::blueprint::ValueSpec;

type DynError = Box<dyn Error>;

pub fn init(matches: &ArgMatches) -> Result<(), DynError> {
    // Parse CLI arguments.
    let template = matches.value_of("template").unwrap();
    let name = matches.value_of("name").unwrap();
    let output_dir = Path::new(matches.value_of("dir").unwrap_or(name));

    let values = matches.values_of("value");

    let mut values: HashMap<String, String> = match values {
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

    check_defaults(&mut values, &blueprint);

    prompt_for_values(&mut values, &blueprint);

    let mustache = templating::Mustache::new();

    blueprint.render(&mustache, &values, &output_dir)?;

    Ok(())
}

fn check_defaults(values: &mut HashMap<String, String>, blueprint: &Blueprint) {
    for value in blueprint.values() {
        if let None = values.get::<str>(&value.name) {
            if let Some(default) = &value.default {
                let key = value.name.clone();
                values.insert(key, default.clone());
            }
        }
    }
}

fn prompt_for_values(values: &mut HashMap<String, String>, blueprint: &Blueprint) {
    for value in blueprint.values() {
        if let None = values.get::<str>(&value.name) {
            if value.required {
                prompt_for_value(values, value);
            }
        }
    }
}

fn prompt_for_value(values: &mut HashMap<String, String>, value: &ValueSpec) {
    print!("{}: ", value.description);
    io::stdout().flush();
    let line: String = read!("{}\n");
    let key = value.name.clone();
    values.insert(key, line);
}

fn parse_value(s: &str) -> Result<(String, String), String> {
    let pos = s.find(":")
        .ok_or(format!("Invalid value `{}`", s))?;

    let mut result = s.split_at(pos);
    result.1 = &result.1[1..];

    Ok((result.0.to_string(), result.1.to_string()))
}

#[test]
fn correct_values_are_parsed_correctly() {
    let (foo, bar) = parse_value("foo:bar").unwrap();

    assert_eq!(foo, "foo");
    assert_eq!(bar, "bar");
}