use std::collections::HashMap;
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

use anyhow::anyhow;
use log::{debug, error, info};
use text_io::read;
use thiserror::Error;

use crate::blueprint::{Blueprint, BlueprintInitError, RendrConfig, ValueSpec, Values};
use crate::templating::tmplpp::{self, Template};
use crate::templating::Mustache;

pub struct Project<'p> {
    path: &'p Path,
    config: RendrConfig,
    blueprint: Blueprint,
}

impl<'p> Project<'p> {
    pub fn new(path: &'p impl AsRef<Path>, blueprint: Blueprint) -> Result<Self, ProjectError> {
        let path = path.as_ref();

        let rendr_file = path.join(Path::new(".rendr.yaml"));
        if !rendr_file.exists() {
            error!(
                "This directory does not appear to be a Rendr project: no .rendr.yaml file found"
            );
            error!("  Expected file at {}", path.display());
            panic!("Project metadata not available");
        }

        let yaml = fs::read_to_string(rendr_file)?;
        let config = serde_yaml::from_str(&yaml)?;

        Ok(Self {
            path,
            config,
            blueprint,
        })
    }

    /// Get a path to the given file within the project.
    pub fn path(&self, p: impl AsRef<Path>) -> PathBuf {
        self.path.join(p)
    }

    pub fn values(&self) -> &Values {
        self.config.values()
    }

    pub fn config(&self) -> &RendrConfig {
        &self.config
    }

    pub fn blueprint(&self) -> &Blueprint {
        &self.blueprint
        // let relative_source = &self.path.join(&self.config.source);
        // debug!("Locating blueprint source, checking if relative source exists: {}", relative_source.display());
        // match relative_source.exists() {
        //     true => Blueprint::new(relative_source.as_os_str().to_str().unwrap(), None),
        //     false => Blueprint::new(&self.config.source.as_str(), None),
        // }
    }

    pub fn validate(&self) -> Result<(), ValidationError> {
        let values = self.values();

        for file in self.blueprint.files() {
            let file = file?;
            let rel_path = file.path_from_template_root();

            if !self.blueprint.is_excluded(rel_path) && !file.path().is_dir() {
                let raw_template = std::fs::read_to_string(file.path())
                    .map_err(|e| ValidationError::TemplateReadError(e))?;
                let template = Template::from_str(&raw_template)?;

                let generated_contents =
                    std::fs::read_to_string(PathBuf::from(self.path).join(rel_path))
                        .map_err(|e| ValidationError::ProjectFileReadError(e))?;

                if !template.validate_generated_output(&values, &generated_contents) {
                    return Err(ValidationError::MatchError(rel_path.to_owned()).into());
                }
            }
        }

        Ok(())
    }

    pub fn upgrade(
        &mut self,
        new_blueprint_source: Option<&str>,
        values: Values,
        dry_run: bool,
    ) -> Result<(), UpgradeError> {
        // Use custom blueprint source if provided on command line
        if let Some(blueprint_source) = new_blueprint_source {
            self.blueprint
                .set_source(blueprint_source)
                .map_err(|e| UpgradeError::BlueprintInitError(e))?;
        }

        if self.blueprint.metadata.editable_templates {
            self.upgrade_blueprint_with_templates(dry_run)
        } else {
            self.upgrade_blueprint_with_scripts(values, dry_run)
        }
    }

    pub fn upgrade_blueprint_with_templates(&self, dry_run: bool) -> Result<(), UpgradeError> {
        let values = self.values();

        for file in self.blueprint.files() {
            let file = file?;
            let rel_path = file.path_from_template_root();

            if self.blueprint.is_excluded(rel_path) || file.path().is_dir() {
                continue;
            }

            let raw_template = std::fs::read_to_string(file.path())
                .map_err(|e| UpgradeError::OldTemplateReadError(e))?;
            let template = Template::from_str(&raw_template)
                .map_err(|e| UpgradeError::OldTemplateParseError(e))?;

            let new_template =
                std::fs::read_to_string(self.blueprint.path().join("template").join(rel_path))
                    .map_err(|e| UpgradeError::NewTemplateReadError(e))?;
            let new_template = Template::from_str(&new_template)
                .map_err(|e| UpgradeError::NewTemplateParseError(e))?;

            let generated_file_path = PathBuf::from(self.path).join(rel_path);
            let generated_contents = std::fs::read_to_string(&generated_file_path)
                .map_err(|e| UpgradeError::ProjectFileReadError(e))?;

            if !template.validate_generated_output(&values, &generated_contents) {
                return Err(UpgradeError::MatchError(rel_path.to_owned()).into());
            }

            let new_content = template.upgrade_to(&new_template, &values, &generated_contents);

            info!(
                "Rendering template {} with content:\n{}",
                rel_path.display(),
                new_content
            );
            if dry_run {
                continue;
            }

            std::fs::write(&generated_file_path, new_content)
                .map_err(|e| UpgradeError::ProjectFileUpgradeError(e))?;
        }

        Ok(())
    }

    pub fn upgrade_blueprint_with_scripts(
        &self,
        cli_values: Values,
        dry_run: bool,
    ) -> Result<(), UpgradeError> {
        debug!("Upgrade dry run mode: {}", dry_run);

        let config = &self.config();
        let blueprint = &self.blueprint;

        info!("{}", blueprint);

        // Check if blueprint version can be upgraded
        if blueprint.metadata.version == config.version {
            info!(
                "Project is already on the latest blueprint version (v{})",
                config.version
            );
            return Ok(());
        } else if blueprint.metadata.version < config.version {
            info!("Project is on a newer version of the blueprint. Something might be wrong.");
            info!("  Project version:   {}", config.version);
            info!("  Blueprint version: {}", blueprint.metadata.version);
            panic!("Canceling upgrade");
        }

        info!(
            "Upgrading project from blueprint version {}",
            blueprint.metadata.version
        );

        // Initialize values with blueprint defaults
        let mut values: HashMap<_, _> = blueprint.default_values().collect();

        // Add values from original project generation
        let config_values: HashMap<&str, &str> = config.values().map_str();
        values.extend(config_values);

        // If some values were provided via CLI arguments, merge those in
        values.extend(cli_values.map_str());

        // Figure out which required values are still missing
        let missing_values = blueprint
            .required_values()
            .filter(|v| values.get::<str>(&v.name).is_none());

        // Prompt for the missing values and collect them
        let prompt_values_owned: Vec<_> = prompt_for_values(missing_values).collect();

        // Merge the values from prompts in
        let prompt_values: Vec<_> = prompt_values_owned
            .iter()
            .map(|(k, v)| (*k, v.as_str()))
            .collect();
        values.extend(prompt_values);

        // Update the target version, inserting if it does not exist for some reason
        let target_version = blueprint.metadata.version.to_string();
        let v = values.entry("version").or_insert(target_version.as_str());
        *v = target_version.as_str();

        info!("Rendering blueprint with values:");
        for (k, v) in values.clone() {
            info!("- {}: {}", k, v);
        }

        // Render new templates
        let mustache = Mustache::new();
        blueprint
            .render_upgrade(
                &mustache,
                &values.into(),
                &self.path,
                &config.source,
                dry_run,
            )
            .map_err(|e| UpgradeError::RenderError(anyhow!("error rendering upgrade: {}", e)))?;

        Ok(())
    }
}

// TODO move this code to a common spot, copied from init.rs
type ValueFromPrompt<'s> = (&'s str, String);

fn prompt_for_values<'s>(
    values: impl Iterator<Item = &'s ValueSpec>,
) -> impl Iterator<Item = ValueFromPrompt<'s>> {
    values.map(prompt_for_value)
}

fn prompt_for_value(value: &ValueSpec) -> ValueFromPrompt<'_> {
    print!("{}: ", value.description);
    io::stdout().flush().unwrap();
    (&value.name, read!("{}\n"))
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
pub enum UpgradeError {
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

    #[error("error upgrading a project file")]
    ProjectFileUpgradeError(#[source] std::io::Error),

    #[error("the generated file {0} doesn't match the blueprint")]
    MatchError(PathBuf),

    #[error("error rendering upgrade")]
    RenderError(#[from] anyhow::Error),
}
