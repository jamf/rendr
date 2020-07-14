use std::error::Error;
use std::collections::HashMap;
use std::path::Path;
use std::io::{self, Write};

use clap::ArgMatches;
use text_io::read;
use log::info;

use rendr::templating;
use rendr::blueprint::Blueprint;
use rendr::blueprint::ValueSpec;

type DynError = Box<dyn Error>;

pub fn init(args: &ArgMatches) -> Result<(), DynError> {
    // Parse CLI arguments.
    let blueprint_path = args.value_of("blueprint").unwrap();
    let name = args.value_of("name").unwrap();
    let output_dir = Path::new(args.value_of("dir").unwrap_or(name));

    // Attempt to read the provided blueprint.
    let blueprint = Blueprint::new(blueprint_path)?;

    println!("{}", blueprint);

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

    info!("Output directory: {:?}. Creating your new scaffold...", &output_dir);

    let mustache = templating::Mustache::new();

    blueprint.render(&mustache, &values, &output_dir)?;
    info!("Success. Enjoy!");

    Ok(())
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
