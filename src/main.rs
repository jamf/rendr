mod templating;

use std::collections::HashMap;
use std::fs;
use std::path::Path;

use clap::{App, ArgMatches, load_yaml};
use git2::Repository;

fn main() {
    let yaml = load_yaml!("cli.yml");
    let matches = App::from_yaml(yaml).get_matches();

    if let Some(matches) = matches.subcommand_matches("init") {
        init(matches);
    }
}

fn init(matches: &ArgMatches) {
    let template = matches.value_of("template").unwrap();
    let name = matches.value_of("name").unwrap();
    let dir = matches.value_of("dir").unwrap_or(name);

    let values = matches.values_of("value").map(|vs| vs.map(parse_value));

    let values: HashMap<&str, &str> = match values {
        Some(values) => values.collect(),
        None         => HashMap::new(),
    };

    println!(
        "Generating project {} from template {}. Directory: {}",
        name,
        template,
        dir,
    );

    let _repo = match Repository::clone(template, dir) {
        Ok(repo) => repo,
        Err(e)   => panic!("failed to init: {}", e),
    };

    fs::create_dir("output").unwrap();

    let output_dir = Path::new("output");

    let blueprint_dir = format!("{}/blueprint", dir);

    if output_dir.is_dir() {
        for entry in fs::read_dir(&blueprint_dir).unwrap() {
            let entry = entry.unwrap();
            let path = entry.path();
            if path.is_file() {
                let filename = path.file_name().unwrap().to_str().unwrap();
                let contents = fs::read_to_string(&path).unwrap();

                let contents = templating::render_template(&contents, &values);

                fs::write(format!("output/{}", filename), &contents).unwrap();
            }
        }
    }
}

fn parse_value(s: &str) -> (&str, &str) {
    let pos = s.find(":").unwrap();
    s.split_at(pos)
}