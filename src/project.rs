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
                    .map_err(|e| ValidationError::ProjectFileReadError(e))?;

                if !template.validate_generated_output(&values, &generated_contents) {
                    return Err(ValidationError::MatchError(rel_path.to_owned()).into());
                }
            }
        }

        Ok(())
    }

    pub fn update(&self, new_blueprint: &Blueprint) -> Result<(), UpdateError> {
        let old_blueprint = self.blueprint()?;
        let values = self.values();

        for file in old_blueprint.files() {
            let file = file?;
            let rel_path = file.path_from_template_root();

            if !old_blueprint.is_excluded(rel_path) && !file.path().is_dir() {
                let raw_template = std::fs::read_to_string(file.path())
                    .map_err(|e| UpdateError::OldTemplateReadError(e))?;
                let template = Template::from_str(&raw_template)
                    .map_err(|e| UpdateError::OldTemplateParseError(e))?;

                let new_template = std::fs::read_to_string(
                    new_blueprint.path()
                        .join("template")
                        .join(rel_path)
                ).map_err(|e| UpdateError::NewTemplateReadError(e))?;
                let new_template = Template::from_str(&new_template)
                    .map_err(|e| UpdateError::NewTemplateParseError(e))?;

                let generated_file_path = PathBuf::from(self.path).join(rel_path);
                let generated_contents = std::fs::read_to_string(&generated_file_path)
                    .map_err(|e| UpdateError::ProjectFileReadError(e))?;

                if !template.validate_generated_output(&values, &generated_contents) {
                    return Err(UpdateError::MatchError(rel_path.to_owned()).into());
                }

                let new_content = template.upgrade_to(&new_template, &values, &generated_contents);
                std::fs::write(&generated_file_path, new_content)
                    .map_err(|e| UpdateError::ProjectFileUpdateError(e))?;
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
    ProjectFileReadError(#[source] std::io::Error),

    #[error("the generated file {0} doesn't match the blueprint")]
    MatchError(PathBuf),
}

#[derive(Error, Debug)]
pub enum UpdateError {
    #[error("error initializing the project's current blueprint")]
    BlueprintInitError(#[from] BlueprintInitError),

    #[error("error traversing the blueprint")]
    WalkDirError(#[from] walkdir::Error),

    #[error("error reading the original template")]
    OldTemplateReadError(#[source] std::io::Error),

    #[error("error parsing the original template")]
    OldTemplateParseError(#[source] tmplpp::TemplateParseError),

    #[error("error reading the new template")]
    NewTemplateReadError(#[source] std::io::Error),

    #[error("error parsing the new template")]
    NewTemplateParseError(#[source] tmplpp::TemplateParseError),

    #[error("error reading a project file")]
    ProjectFileReadError(#[source] std::io::Error),

    #[error("error updating a project file")]
    ProjectFileUpdateError(#[source] std::io::Error),

    #[error("the generated file {0} doesn't match the blueprint")]
    MatchError(PathBuf),
}