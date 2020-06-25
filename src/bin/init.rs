use std::error::Error;
use std::collections::HashMap;
use std::path::Path;

use clap::ArgMatches;

use express::templating;
use express::blueprint::Blueprint;

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

    let blueprint = Blueprint::from_remote_repo(template)?;

    let mustache = templating::Mustache::new();

    blueprint.render(&mustache, &values, &output_dir)?;

    Ok(())
}

// Parse a string of "key:value" form into a tuple of (key, value).
fn parse_value(s: &str) -> Result<(&str, &str), String> {
    let pos = s.find(":")
        .ok_or(format!("Invalid value `{}`", s))?;

    let mut result = s.split_at(pos);
    result.1 = &result.1[1..];

    Ok(result)
}

#[test]
fn correct_values_are_parsed_correctly() {
    let (foo, bar) = parse_value("foo:bar").unwrap();

    assert_eq!(foo, "foo");
    assert_eq!(bar, "bar");
}