mod templating;

use std::fs;
use std::path::Path;

use clap::{App, ArgMatches, load_yaml};
use git2::Repository;
use mustache::MapBuilder;

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

                let contents = render_template(&contents);

                fs::write(format!("output/{}", filename), &contents).unwrap();
            }
        }
    }
}

fn render_template(s: &str) -> String {
    let template = mustache::compile_str(s).unwrap();

    let data = MapBuilder::new()
        .insert_str("name", "Venus")
        .insert_str("version", "0.1.0")
        .insert_str("foobar", "42")
        .build();

    template.render_data_to_string(&data).unwrap()
}
