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
}
