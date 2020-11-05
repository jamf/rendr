pub mod source;
mod values;

use std::clone::Clone;
use std::error::Error;
use std::fmt::Display;
use std::fmt::Formatter;
use std::fs;
use std::io::Write;
use std::path::Path;
use std::path::PathBuf;
use std::process::{Command, Stdio};

use log::{debug, info};
use serde::{Deserialize, Serialize};
use serde_yaml;
use thiserror::Error;
use walkdir::{DirEntry, WalkDir};

use crate::blueprint::source::BlueprintSourceError;
use crate::templating::TemplatingEngine;
use crate::Pattern;
use source::Source;
pub use values::Values;

type DynError = Box<dyn Error>;

#[derive(Clone)]
pub struct BlueprintAuth {
    user: Option<String>,
    password: Option<String>,
    ssh_key: Option<String>,
}

impl BlueprintAuth {
    pub fn new(user: Option<String>, password: Option<String>, ssh_key: Option<String>) -> Self {
        BlueprintAuth {
            user,
            password,
            ssh_key,
        }
    }
}

pub struct Blueprint {
    pub auth: Option<BlueprintAuth>,
    pub metadata: BlueprintMetadata,
    pub source: Source,
    pub post_script: Option<Script>,
}

impl Blueprint {
    pub fn new(source: &str, auth: Option<BlueprintAuth>) -> Result<Blueprint, BlueprintInitError> {
        debug!("Initializing blueprint from source {}", source);

        let source = Source::new(source, auth.clone())?;
        let metadata_path = source.path().join("metadata.yaml");

        debug!(
            "Loading blueprint metadata from {}",
            metadata_path.display()
        );
        let meta_raw = fs::read_to_string(metadata_path)?;

        debug!("Loaded blueprint metadata: {}", meta_raw);
        let metadata = serde_yaml::from_str(&meta_raw)?;

        let mut blueprint = Blueprint {
            auth,
            metadata,
            source,
            post_script: None,
        };

        blueprint.find_scripts()?;

        Ok(blueprint)
    }

    pub fn path(&self) -> &Path {
        self.source.path()
    }

    pub fn set_source(&mut self, source: &str) -> Result<(), BlueprintInitError> {
        self.source = Source::new(source, self.auth.clone())?;

        Ok(())
    }

    fn find_scripts(&mut self) -> Result<(), BlueprintInitError> {
        self.post_script = self.find_script("post-render.sh")?;

        Ok(())
    }

    fn find_script(&self, script: &str) -> Result<Option<Script>, BlueprintInitError> {
        let mut script_path = PathBuf::new();
        script_path.push(
            self.source
                .path()
                .canonicalize()
                .map_err(|e| BlueprintInitError::ScriptLookupError(e))?,
        );
        script_path.push("scripts");
        script_path.push(format!("{}", script));

        if !script_path.exists() {
            debug!(
                "No {} script found in blueprint scripts directory - skipping",
                script
            );
            return Ok(None);
        }

        Ok(Some(Script::new(script, script_path)))
    }

    pub fn is_git_init_enabled(&self) -> bool {
        self.metadata.git_init
    }

    pub fn values(&self) -> impl Iterator<Item = &ValueSpec> {
        self.metadata.values.iter()
    }

    pub fn default_values(&self) -> impl Iterator<Item = (&str, &str)> {
        self.values()
            .filter(|v| v.default.is_some())
            .map(|v| (v.name.as_str(), v.default.as_ref().unwrap().as_str()))
    }

    pub fn required_values(&self) -> impl Iterator<Item = &ValueSpec> {
        self.values().filter(|v| v.required)
    }

    pub fn files(&self) -> impl Iterator<Item = Result<File, walkdir::Error>> {
        let template_root = self.source.path().join("template");
        Files::new(&template_root)
    }

    pub fn is_excluded<P: AsRef<Path>>(&self, file: P) -> bool {
        self.metadata
            .exclusions
            .iter()
            .find(|pattern| pattern.matches_path(file.as_ref()))
            .is_some()
    }

    pub fn render<'s, TE: TemplatingEngine>(
        &self,
        engine: &TE,
        values: &Values,
        output_dir: &Path,
        dry_run: bool,
    ) -> Result<(), DynError> {
        // Create our output directory if it doesn't exist yet.
        debug!("Creating root project dir {:?}", &output_dir);
        if !output_dir.is_dir() {
            fs::create_dir(output_dir)?;
        }

        for file in self.files() {
            let file = file?;
            let path = file.path();
            let output_path = output_dir.join(file.path_from_template_root());

            if path.is_file() {
                debug!("Found file {:?}", &file.path_from_template_root);

                if self.is_excluded(&file.path_from_template_root) {
                    debug!(
                        "Copying {:?} without templating.",
                        &file.path_from_template_root
                    );
                    fs::copy(path, output_path)?;
                } else {
                    debug!(
                        "Using template {:?} to render {:?}",
                        &file.path_from_template_root, &output_path
                    );
                    let contents = fs::read_to_string(&path)?;
                    let contents = engine.render_template(&contents, values.clone())?;
                    fs::write(output_path, &contents)?;
                }
            } else if path.is_dir() {
                if !output_path.is_dir() {
                    debug!("Creating directory {:?}", &file.path_from_template_root);
                    fs::create_dir(&output_path)?;
                }
            }
        }

        if let Some(post_script) = &self.post_script {
            post_script.run(output_dir, &values)?;
        }

        let source = self.source.to_string(output_dir);
        self.generate_rendr_file(&source, &output_dir, &values, dry_run)?;

        Ok(())
    }

    pub fn render_upgrade<TE: TemplatingEngine>(
        &self,
        engine: &TE,
        values: &Values,
        output_dir: &Path,
        source: &str,
        dry_run: bool,
    ) -> Result<(), DynError> {
        info!("Upgrading to blueprint version {}", &self.metadata.version);
        debug!("Root project dir {:?}", &output_dir);

        for file in self.files() {
            let file = file?;
            let path = file.path();
            let output_path = output_dir.join(file.path_from_template_root());

            if path.is_file() {
                if self.is_excluded(&file.path_from_template_root) {
                    debug!(
                        "Copying {:?} without templating.",
                        &file.path_from_template_root
                    );
                    if !dry_run {
                        fs::copy(path, output_path)?;
                    }
                } else if output_path.exists() {
                    debug!(
                        "Skipping {:?}, file already exists",
                        &file.path_from_template_root
                    );
                } else {
                    debug!(
                        "Using template {:?} to render {:?}",
                        &file.path_from_template_root, &output_path
                    );
                    let contents = fs::read_to_string(&path)?;
                    let contents = engine.render_template(&contents, values.clone())?;
                    if !dry_run {
                        fs::write(output_path, &contents)?;
                    }
                }
            } else if path.is_dir() {
                if !output_path.is_dir() {
                    debug!("Creating directory {:?}", &file.path_from_template_root);
                    if !dry_run {
                        fs::create_dir(&output_path)?;
                    }
                }
            }
        }

        let scripts = self.get_upgrade_scripts();
        self.run_upgrade_scripts(scripts, output_dir, values, dry_run)?;
        self.generate_rendr_file(&source, &output_dir, &values, dry_run)?;

        Ok(())
    }

    fn generate_rendr_file<'s>(
        &self,
        source: &str,
        output_dir: &Path,
        values: &Values,
        dry_run: bool,
    ) -> Result<(), DynError> {
        debug!("Generating .rendr.yaml file:");
        debug!("  source: {}", source);
        debug!("  output_dir: {}", output_dir.display());
        debug!("  values: {:?}", values);

        let path = output_dir.join(Path::new(".rendr.yaml"));
        let config = RendrConfig::new(source.to_string().clone(), &self.metadata, values.clone());
        let yaml = serde_yaml::to_string(&config)?;

        if !dry_run {
            let mut file = std::fs::File::create(path)?;
            file.write_all(yaml.as_bytes())?;
        }

        Ok(())
    }

    pub fn get_upgrade_scripts(&self) -> Vec<&UpgradeSpec> {
        self.metadata
            .upgrades
            .iter()
            .filter(|it| it.version == self.metadata.version)
            .collect()
    }

    fn run_upgrade_scripts(
        &self,
        scripts: Vec<&UpgradeSpec>,
        output_dir: &Path,
        values: &Values,
        dry_run: bool,
    ) -> Result<(), DynError> {
        let target_version = self.metadata.version;
        debug!(
            "Running {} upgrade script(s) for version {}",
            scripts.len(),
            target_version
        );

        for script in scripts {
            if script.version == target_version {
                if let Some(mut s) = self.find_script(&script.script).unwrap() {
                    s.executable = Some(String::from(&script.executable));
                    println!("Running upgrade script: {}", s.name);
                    if !dry_run {
                        s.run(output_dir, values)?;
                    }
                }
            }
        }

        Ok(())
    }
}

#[derive(Error, Debug)]
pub enum BlueprintInitError {
    #[error("error finding blueprint")]
    SourceError(#[from] BlueprintSourceError),

    #[error("error reading blueprint metadata")]
    MetadataReadError(#[from] std::io::Error),

    #[error("error parsing blueprint metadata")]
    MetadataParseError(#[from] serde_yaml::Error),

    #[error("error looking up blueprint scripts")]
    ScriptLookupError(#[source] std::io::Error),
}

#[derive(Serialize, Deserialize)]
pub struct RendrConfig {
    pub name: String,
    pub version: u32,
    pub author: String,
    pub description: String,
    pub source: String,
    values: Values,
}

impl RendrConfig {
    fn new(source: String, metadata: &BlueprintMetadata, values: Values) -> Self {
        RendrConfig {
            name: metadata.name.clone(),
            version: metadata.version,
            author: metadata.author.clone(),
            description: metadata.description.clone(),
            source,
            values: values,
        }
    }

    pub fn values(&self) -> &Values {
        &self.values
    }

    pub fn blueprint(&self) -> Result<Blueprint, BlueprintInitError> {
        Blueprint::new(&self.source, None)
    }

    pub fn load(metadata_file: &PathBuf) -> Result<Option<RendrConfig>, DynError> {
        let yaml = fs::read_to_string(metadata_file)?;
        let config: RendrConfig = serde_yaml::from_str(&yaml)?;

        Ok(Some(config))
    }
}

impl Display for RendrConfig {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        writeln!(f, "name: {}", &self.name)?;
        writeln!(f, "version: {}", &self.version)?;
        writeln!(f, "description: {}", &self.description)?;
        writeln!(f, "author: {}", &self.author)?;
        writeln!(f, "source: {}", &self.source)?;
        writeln!(f, "values:")?;
        for (name, value) in self.values.iter() {
            writeln!(f, "- name: {}", name)?;
            writeln!(f, "  value: {}", value)?;
        }

        Ok(())
    }
}

pub struct Files {
    walkdir: walkdir::IntoIter,
    template_root: PathBuf,
}

impl Files {
    fn new(template_root: &Path) -> Self {
        Files {
            walkdir: WalkDir::new(template_root).into_iter(),
            template_root: template_root.to_path_buf(),
        }
    }
}

impl Iterator for Files {
    type Item = walkdir::Result<File>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(next) = self.walkdir.next() {
            match next {
                Ok(entry) => return Some(Ok(File::new(&self.template_root, entry))),
                Err(e) => return Some(Err(e)),
            }
        }

        None
    }
}

pub struct File {
    dir_entry: DirEntry,
    path_from_template_root: PathBuf,
}

impl File {
    fn new<P: AsRef<Path>>(template_root: P, dir_entry: DirEntry) -> Self {
        let depth = template_root.as_ref().components().count();
        let path_from_template_root = dir_entry
            .path()
            .components()
            .skip(depth)
            .map(|c| c.as_os_str())
            .fold(PathBuf::new(), |a, b| a.join(b));

        File {
            dir_entry,
            path_from_template_root: path_from_template_root.to_path_buf(),
        }
    }

    pub fn path(&self) -> &Path {
        self.dir_entry.path()
    }

    pub fn path_from_template_root(&self) -> &Path {
        &self.path_from_template_root
    }
}

pub struct Script {
    executable: Option<String>,
    name: String,
    path: PathBuf,
}

impl Script {
    fn new(name: &str, path: PathBuf) -> Self {
        Script {
            executable: None,
            name: name.to_string(),
            path: path,
        }
    }

    fn run<'v>(&self, working_dir: &Path, values: &Values) -> Result<(), DynError> {
        info!("Running blueprint script: {}", &self.name);

        #[cfg(debug)]
        debug!("  Blueprint script executable: {:?}", &self.executable);
        debug!("  Blueprint script full path: {:?}", &self.path);
        debug!("  Blueprint script working dir: {:?}", working_dir);

        let values_flags = values
            .iter()
            .map(|i| format!("--value {}={}", i.0, i.1))
            .collect::<Vec<String>>()
            .join(" ");
        let command = match &self.executable {
            Some(executable) => format!("{} {} {}", executable, &self.path.display(), values_flags),
            None => format!("{}", &self.path.display()),
        };

        debug!("  Executing command: {}", command);
        let output = Command::new("sh")
            .arg("-c")
            .arg(command)
            .envs(values.map())
            .current_dir(working_dir)
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .output()
            .expect("failed to execute script");

        debug!("Status: {}", output.status);

        if !output.status.success() {
            let e = ScriptError::new(output.status.code(), String::from_utf8(output.stderr)?);
            return Err(e.into());
        }

        Ok(())
    }
}

#[derive(Debug)]
struct ScriptError {
    status: Option<i32>,
    msg: String,
}

impl ScriptError {
    fn new(status: Option<i32>, msg: String) -> Self {
        ScriptError { status, msg }
    }
}

impl Error for ScriptError {}

impl Display for ScriptError {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        match self.status {
            Some(status) => write!(f, "Script failed with status {}", status)?,
            None => write!(f, "Script failed, but didn't exit!")?,
        }

        Ok(())
    }
}

impl Display for Blueprint {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        writeln!(f, "{} v{}", &self.metadata.name, &self.metadata.version)?;
        writeln!(f, "{}", &self.metadata.author)?;
        writeln!(f, "{}", &self.metadata.description)?;

        Ok(())
    }
}

#[derive(Serialize, Deserialize)]
pub struct BlueprintMetadata {
    pub name: String,
    pub version: u32,
    pub author: String,
    pub description: String,
    #[serde(default)]
    pub editable_templates: bool,
    pub values: Vec<ValueSpec>,
    #[serde(default)]
    pub exclusions: Vec<Pattern>,
    #[serde(alias = "git-init")]
    #[serde(default)]
    pub git_init: bool,
    #[serde(default)]
    pub upgrades: Vec<UpgradeSpec>,
}

#[derive(Serialize, Deserialize)]
pub struct ValueSpec {
    pub name: String,
    pub description: String,
    pub default: Option<String>,
    #[serde(default)]
    pub required: bool,
}

#[derive(Serialize, Deserialize)]
pub struct UpgradeSpec {
    pub version: u32,
    pub script: String,
    pub executable: String,
}

impl PartialEq for ValueSpec {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

impl Eq for ValueSpec {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::blueprint::Blueprint;
    use crate::templating::Tmplpp;
    use std::collections::HashMap;
    use std::fs;
    use tempdir::TempDir;

    #[test]
    fn parse_example_blueprint_metadata() {
        let blueprint = Blueprint::new("test_assets/example_blueprint", None).unwrap();

        assert_eq!(blueprint.metadata.name, "example-blueprint");
        assert_eq!(blueprint.metadata.version, 1);
        assert_eq!(
            blueprint.metadata.author,
            "Brian S. <brian.stewart@jamf.com>, Tomasz K. <tomasz.kurcz@jamf.com>"
        );
        assert_eq!(
            blueprint.metadata.description,
            "Just an example blueprint for `rendr`."
        );
    }

    #[test]
    fn render_example_blueprint() {
        let blueprint = Blueprint::new("test_assets/example_blueprint", None).unwrap();

        let output_dir = TempDir::new("my-project").unwrap();

        let engine = Tmplpp::new();

        blueprint
            .render(&engine, &test_values(), output_dir.path(), false)
            .unwrap();

        let test = fs::read_to_string(output_dir.path().join("test.yaml")).unwrap();
        let another_test = fs::read_to_string(output_dir.path().join("another-test.yaml")).unwrap();

        assert!(test.find("name: my-project").is_some());
        assert!(test.find("version: 1").is_some());
        assert!(another_test.find("stuff: stuff").is_some());
        assert!(another_test.find("version: 1").is_some());
    }

    #[test]
    fn render_example_blueprint_recursive() {
        let blueprint = Blueprint::new("test_assets/example_blueprint", None).unwrap();

        let output_dir = TempDir::new("my-project").unwrap();

        let engine = Tmplpp::new();

        blueprint
            .render(&engine, &test_values(), output_dir.path(), false)
            .unwrap();

        let test = fs::read_to_string(output_dir.path().join("dir/test.yaml")).unwrap();

        assert!(test.find("name: my-project").is_some());
        assert!(test.find("version: 1").is_some());
    }

    #[test]
    fn exclusions_work() {
        let blueprint = Blueprint::new("test_assets/example_blueprint", None).unwrap();

        let output_dir = TempDir::new("my-project").unwrap();

        let engine = Tmplpp::new();

        blueprint
            .render(&engine, &test_values(), output_dir.path(), false)
            .unwrap();

        let excluded_file = fs::read_to_string(output_dir.path().join("excluded_file")).unwrap();

        assert!(excluded_file.find("{{ name }}").is_some());
    }

    #[test]
    fn glob_exclusions_work() {
        let blueprint = Blueprint::new("test_assets/example_blueprint", None).unwrap();

        let output_dir = TempDir::new("my-project").unwrap();

        let engine = Tmplpp::new();

        blueprint
            .render(&engine, &test_values(), output_dir.path(), false)
            .unwrap();

        let excluded_file1 =
            fs::read_to_string(output_dir.path().join("excluded_files/foo")).unwrap();
        let excluded_file2 =
            fs::read_to_string(output_dir.path().join("excluded_files/bar")).unwrap();

        assert!(excluded_file1.find("{{ name }}").is_some());
        assert!(excluded_file2.find("{{ name }}").is_some());
    }

    #[test]
    fn script_can_be_run_successfully() {
        let script = Script::new(
            "some script",
            PathBuf::from("test_assets/scripts/hello_world.sh"),
        );

        script.run(Path::new("."), &Values::new()).unwrap();
    }

    #[test]
    fn running_failing_script_returns_an_error() {
        let script = Script::new(
            "some script",
            PathBuf::from("test_assets/scripts/failing.sh"),
        );

        if let Ok(()) = script.run(Path::new("."), &Values::new()) {
            panic!("The failing script didn't cause an error!");
        }
    }

    #[test]
    fn blueprint_post_script_is_found_and_run() {
        let blueprint = Blueprint::new("test_assets/example_blueprint_with_scripts", None).unwrap();

        let output_dir = TempDir::new("my-project").unwrap();

        let engine = Tmplpp::new();

        blueprint
            .render(&engine, &test_values(), output_dir.path(), false)
            .unwrap();

        let script_output = fs::read_to_string(output_dir.path().join("script_output.md")).unwrap();

        assert_eq!(script_output.as_str(), "something123");
    }

    // Test helpers
    fn test_values() -> Values {
        vec![
            ("name", "my-project"),
            ("version", "1"),
            ("foobar", "stuff"),
        ]
        .iter()
        .cloned()
        .collect::<HashMap<_, _>>()
        .into()
    }
}
