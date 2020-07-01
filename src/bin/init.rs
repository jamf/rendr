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

    let mut values: HashMap<&str, &str> = match values {
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

    let prompt_values = prompt_for_values(&mut values, &blueprint);
    let prompt_hash: HashMap<&str, &str> = prompt_values.iter()
        .map(|(k, v)| (*k, v.as_str()))
        .collect();

    values.extend(prompt_hash);

    let mustache = templating::Mustache::new();

    blueprint.render(&mustache, &values, &output_dir)?;

    Ok(())
}

fn check_defaults<'s>(values: &mut HashMap<&'s str, &'s str>, blueprint: &'s Blueprint) {
    for value in blueprint.values() {
        if let None = values.get::<str>(&value.name) {
            if let Some(default) = &value.default {
                let key = &value.name;
                values.insert(key, default);
            }
        }
    }
}

fn prompt_for_values<'s>(values: &HashMap<&'s str, &str>, blueprint: &'s Blueprint) -> Vec<(&'s str, String)> {
    blueprint.values()
        .iter()
        .filter(|v| values.get::<str>(&v.name).is_none())   // only take values that aren't yet in `values`
        .filter(|v| v.required)                             // only take required values
        .map(prompt_for_value)
        .collect()
}

fn prompt_for_value(value: &ValueSpec) -> (&str, String) {
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