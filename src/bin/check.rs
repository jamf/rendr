use std::collections::HashMap;
use std::error::Error;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

use clap::ArgMatches;
use git2::{Oid, Repository, IndexAddOption, Signature};
use log::{info, debug};
use text_io::read;
use thiserror::Error;

use rendr::templating::{Tmplpp, tmplpp::Template};
use rendr::blueprint::{Blueprint, ValueSpec};
use rendr::project::Project;

type DynError = Box<dyn Error>;

pub fn check(args: &ArgMatches) -> Result<(), DynError> {
    // Parse CLI arguments.
    let project_path = args.value_of("project").unwrap_or(".");

    // Attempt to parse the provided project.
    let project = Project::new(&project_path)?;
    let values = project.values();
    let blueprint = project.blueprint()?;

    for file in blueprint.files() {
        let file = file?;
        let rel_path = file.path_from_template_root();

        if !blueprint.is_excluded(rel_path) && !file.path().is_dir() {
            let raw_template = std::fs::read_to_string(file.path())?;
            let template = Template::from_str(&raw_template)?;

            let generated_contents = std::fs::read_to_string(PathBuf::from(project_path).join(rel_path))?;

            if !template.validate_generated_output(&values, &generated_contents) {
                return Err(CheckError::ValidationError(rel_path.to_owned()).into());
            }
        }
    }

    Ok(())
}

#[derive(Error, Debug)]
enum CheckError {
    #[error("Generated file doesn't match the template for {0}")]
    ValidationError(PathBuf)
}
