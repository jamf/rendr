use std::path::{Path, PathBuf};
use std::fs;

use thiserror::Error;

use crate::blueprint::{Values, RendrConfig, Blueprint, BlueprintInitError};
use crate::templating::tmplpp::{self, Template};

pub struct Project<'p> {
    path: &'p Path,
    meta: RendrConfig,
}

impl<'p> Project<'p> {
    pub fn new(path: &'p impl AsRef<Path>) -> Result<Self, ProjectError> {
        let path = path.as_ref();
        
        let meta_file = fs::read_to_string(path.join(".rendr.yaml"))?;
        let meta = serde_yaml::from_str(&meta_file)?;

        Ok(Self {
            path,
            meta,
        })
    }

    /// Get a path to the given file within the project.
    pub fn path(&self, p: impl AsRef<Path>) -> PathBuf {
        self.path.join(p)
    }

    pub fn values(&self) -> &Values {
        self.meta.values()
    }

    pub fn blueprint(&self) -> Result<Blueprint, BlueprintInitError> {
        Blueprint::new(
            self.path
                .join(&self.meta.source)
                .as_os_str()
                .to_str()
                .unwrap(),
            None,
        )
    }

    pub fn validate(&self) -> Result<(), ValidationError> {
        let blueprint = self.blueprint()?;
        let values = self.values();

        for file in blueprint.files() {
            let file = file?;
            let rel_path = file.path_from_template_root();

            if !blueprint.is_excluded(rel_path) && !file.path().is_dir() {
                let raw_template = std::fs::read_to_string(file.path())
                    .map_err(|e| ValidationError::TemplateReadError(e))?;
                let template = Template::from_str(&raw_template)?;

                let generated_contents = std::fs::read_to_string(PathBuf::from(self.path).join(rel_path))
                    .map_err(|e| ValidationError::GeneratedFileReadError(e))?;

                if !template.validate_generated_output(&values, &generated_contents) {
                    return Err(ValidationError::MatchError(rel_path.to_owned()).into());
                }
            }
        }

        Ok(())
    }
}

#[derive(Error, Debug)]
pub enum ProjectError {
    #[error("error reading project's metadata")]
    MetaFileError(#[from] std::io::Error),

    #[error("error parsing project's metadata")]
    MetaParseError(#[from] serde_yaml::Error),
}

#[derive(Error, Debug)]
pub enum ValidationError {
    #[error("error initializing the project's blueprint")]
    BlueprintInitError(#[from] BlueprintInitError),

    #[error("error traversing the blueprint")]
    WalkDirError(#[from] walkdir::Error),

    #[error("error reading template")]
    TemplateReadError(#[source] std::io::Error),

    #[error("error parsing template")]
    TemplateParseError(#[from] tmplpp::TemplateParseError),

    #[error("error reading generated file")]
    GeneratedFileReadError(#[source] std::io::Error),

    #[error("the generated file {0} doesn't match the blueprint")]
    MatchError(PathBuf),
}
