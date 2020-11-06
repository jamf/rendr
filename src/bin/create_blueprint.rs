use std::fs;
use std::fs::File;
use std::io::Write;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;

use anyhow::{anyhow, Error};
use clap::ArgMatches;
use log::{debug, info};

use rendr::blueprint::{BlueprintMetadata, ValueSpec};

pub fn create(args: &ArgMatches) -> Result<(), Error> {
    let name = args.value_of("name").unwrap();
    let author = args.value_of("author").unwrap_or("");
    let description = args.value_of("description").unwrap_or("A simple blueprint for Rendr");
    let dir = Path::new(name);

    info!("Creating blueprint '{}'", name);
    if dir.exists() {
        debug!("File or directory already exists at {}. Exiting.", dir.display());
        return Err(anyhow!("directory '{}' already exists", dir.display()));
    }

    debug!("Creating directory {}", dir.display());
    fs::create_dir(Path::new(dir))?;

    let template_dir = dir.join("template");
    let scripts_dir = dir.join("scripts");
    let metadata_path = dir.join("metadata.yaml");

    debug!("Creating directory {}", template_dir.display());
    fs::create_dir(Path::new(&template_dir))?;
    debug!("Creating directory {}", scripts_dir.display());
    fs::create_dir(Path::new(&scripts_dir))?;

    let value1 = ValueSpec {
        name: String::from("name"),
        description: String::from("The app name"),
        required: true,
        default: Option::None,
    };
    let value2 = ValueSpec {
        name: String::from("magic_number"),
        description: String::from("The magic number"),
        required: false,
        default: Option::Some(String::from("42")),
    };
    let values = vec!(value1, value2);

    let config = BlueprintMetadata {
        name: String::from(name),
        version: 1,
        author: String::from(author),
        description: String::from(description),
        editable_templates: false,
        values: values,
        exclusions: Vec::new(),
        git_init: false,
        upgrades: Vec::new(),
    };

    let metadata = serde_yaml::to_string(&config)?;

    debug!("Creating file {} with contents:\n{}", metadata_path.display(), metadata);
    let mut metadata_file = File::create(metadata_path)?;
    metadata_file.write_all(metadata.as_bytes())?;

    let app_path = template_dir.join("app.sh");
    let app_code = "#!/bin/sh

name='{{ name }}'
magic_number='{{ magic_number }}'

echo \"Hello, from $name!
The magic number is $magic_number.\"";
    debug!("Creating file {} with contents:\n{}", app_path.display(), app_code);
    let mut app_file = File::create(app_path)?;
    app_file.write_all(app_code.as_bytes())?;

    let script_path = scripts_dir.join("post-render.sh");
    let script_code = "#!/bin/sh

# The values supplied by the user are available as variables
# in this script and can be used as seen below.

echo \"Running some post-render customizations\"

# Generate a README file (this could be a template, just demonstrating it can be done)
echo \"# $name\" > README.md
echo \"\" >> README.md
echo \"The magic number is $magic_number.\" >> README.md

# Make the app script executable
chmod +x app.sh
";
    debug!("Creating file {} with contents:\n{}", script_path.display(), script_code);
    let mut script_file = File::create(Path::new(&script_path))?;
    script_file.write_all(script_code.as_bytes())?;

    let mut permissions = script_file.metadata()?.permissions();
    permissions.set_mode(0o744);
    fs::set_permissions(script_path, permissions)?;

    let readme_path = dir.join("README.md");
    let readme_text = format!("# Rendr Blueprint: {}

Welcome! This is your new [Rendr](https://github.com/jamf/rendr) blueprint!

As a blueprint author, your next steps are:
1. Create template files in the `template` directory
2. Create any post-render customizations in the `scripts/post-render.sh` script
3. Edit this README to provide usage instructions for your blueprint, like below

## Usage

To create a project from this blueprint, run:

    rendr init --blueprint <path or url> --dir <project name>

Then, run the app:

    cd <project name>
    ./app.sh

", name);
    debug!("Creating file {} with contents:\n{}", readme_path.display(), readme_text);
    let mut readme_file = File::create(readme_path)?;
    readme_file.write_all(readme_text.as_bytes())?;

    info!("Success!");

    Ok(())
}