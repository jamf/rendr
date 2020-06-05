#[macro_use]
extern crate clap;
use clap::{App, ArgMatches};

fn main() {
    let yaml = load_yaml!("cli.yml");
    let matches = App::from_yaml(yaml).get_matches();

    // You can handle information about subcommands by requesting their matches by name
    // (as below), requesting just the name used, or both at the same time
    if let Some(matches) = matches.subcommand_matches("init") {
        init(matches);
    }

        // if matches.is_present("debug") {
        //     println!("Printing debug info...");
        // } else {
        //     println!("Printing normally...");
        // }
}

fn init(matches: &ArgMatches) {
    // Gets a value for config if supplied by user, or defaults to "default.conf"
    let template = matches.value_of("template").unwrap();
    let name = matches.value_of("name").unwrap();
    let dir = matches.value_of("dir").unwrap_or(name);

    println!(
        "Generating project {} from template {}. Directory: {}",
        name,
        template,
        dir,
    );
}