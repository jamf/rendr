use anyhow::Error;
use std::path::PathBuf;

use clap::ArgMatches;
use thiserror::Error;

use rendr::templating::{tmplpp::Template};
use rendr::blueprint::{Blueprint};
use rendr::project::Project;

pub fn update(args: &ArgMatches) -> Result<(), Error> {
    // Parse CLI arguments.
    let project_path = args.value_of("project").unwrap_or(".");
    let new_blueprint = Blueprint::new(args.value_of("blueprint").unwrap(), None)?;

    // Attempt to parse the provided project.
    let project = Project::new(&project_path)?;
    let values = project.values();
    let old_blueprint = project.blueprint()?;

    for file in old_blueprint.files() {
        let file = file?;
        let rel_path = file.path_from_template_root();

        if !old_blueprint.is_excluded(rel_path) && !file.path().is_dir() {
            let raw_template = std::fs::read_to_string(file.path())?;
            let template = Template::from_str(&raw_template)?;
            let new_template = std::fs::read_to_string(new_blueprint.path().join("template").join(rel_path))?;
            let new_template = Template::from_str(&new_template)?;

            let generated_file_path = PathBuf::from(project_path).join(rel_path);
            let generated_contents = std::fs::read_to_string(&generated_file_path)?;

            if !template.validate_generated_output(&values, &generated_contents) {
                return Err(CheckError::ValidationError(rel_path.to_owned()).into());
            }

            let new_content = template.upgrade_to(&new_template, &values, &generated_contents);
            std::fs::write(&generated_file_path, new_content)?;
        }
    }

    Ok(())
}

#[derive(Error, Debug)]
enum CheckError {
    #[error("Generated file doesn't match the template for {0}")]
    ValidationError(PathBuf)
}
