mod source;

use std::collections::HashMap;
use std::env;
use std::error::Error;
use std::fmt::Display;
use std::fmt::Formatter;
use std::fs;
use std::io::Write;
use std::path::Path;
use std::path::PathBuf;
use std::process::{Command, Stdio};

use log::{info, debug};
use serde::{Deserialize, Serialize};
use serde_yaml;
use walkdir::{DirEntry, WalkDir};

use crate::Pattern;
use crate::templating::TemplatingEngine;
use source::Source;

type DynError = Box<dyn Error>;

pub struct Blueprint {
    pub metadata: BlueprintMetadata,
    pub source: Source,
    pub post_script: Option<Script>,
}

impl Blueprint {
    pub fn new(source: &str) -> Result<Blueprint, DynError> {
        let source = Source::new(source)?;

        let meta_raw = fs::read_to_string(source.path().join("metadata.yaml"))?;

        let metadata = serde_yaml::from_str(&meta_raw)?;

        let mut blueprint = Blueprint {
            metadata,
            source,
            post_script: None,
        };

        blueprint.find_scripts()?;

        Ok(blueprint)
    }

    fn find_scripts(&mut self) -> Result<(), DynError> {
        self.post_script = self.find_script("post-render")?;

        Ok(())
    }

    fn find_script(&mut self, script: &str) -> Result<Option<Script>, DynError> {
        let mut script_path = PathBuf::new();
        script_path.push(self.source.path().canonicalize()?);
        script_path.push("scripts");
        script_path.push(format!("{}.sh", script));

        if !script_path.exists() {
            debug!("No {} script found in blueprint scripts directory - skipping", script);
            return Ok(None);
        }

        Ok(Some(Script::new(script, script_path)))
    }

    pub fn is_git_init_enabled(&self) -> bool {
        self.metadata.git_init
    }

    pub fn values(&self) -> impl Iterator<Item=&ValueSpec> {
        self.metadata.values.iter()
    }

    pub fn default_values(&self) -> impl Iterator<Item=(&str, &str)> {
        self.values()
            .filter(|v| v.default.is_some())
            .map(|v| (v.name.as_str(), v.default.as_ref().unwrap().as_str()))
    }

    pub fn required_values(&self) -> impl Iterator<Item=&ValueSpec> {
        self.values()
            .filter(|v| v.required)
    }

    fn files(&self) -> impl Iterator<Item=Result<File, walkdir::Error>> {
        let template_root = self.source.path().join("template");
        Files::new(&template_root)
    }

    fn is_excluded<P: AsRef<Path>>(&self, file: P) -> bool {
        self.metadata.exclusions
            .iter()
            .find(|pattern| pattern.matches_path(file.as_ref()))
            .is_some()
    }

    pub fn render<TE: TemplatingEngine>(
            &self,
            engine: &TE,
            values: &HashMap<&str, &str>,
            output_dir: &Path,
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
                debug!("Found file {:?}", &path);

                if self.is_excluded(&file.path_from_template_root) {
                    debug!("Copying {:?} to {:?} without templating.", &path, &output_path);
                    fs::copy(path, output_path)?;
                }
                else {
                    debug!("Using template {:?} to render {:?}", &path, &output_path);
                    let contents = fs::read_to_string(&path)?;
                    let contents = engine.render_template(&contents, &values)?;
                    fs::write(output_path, &contents)?;
                }
            }
            else if path.is_dir() {
                if !output_path.is_dir() {
                    debug!("Creating directory {:?}", &output_path);
                    fs::create_dir(&output_path)?;
                }
            }
        }

        if let Some(post_script) = &self.post_script {
            post_script.run(output_dir, values)?;
        }

        let source = self.source.to_string(output_dir);
        self.generate_rendr_file(&source, &output_dir, &values)?;

        Ok(())
    }

    pub fn render_upgrade<TE: TemplatingEngine>(
            &self,
            engine: &TE,
            values: &HashMap<&str, &str>,
            output_dir: &Path,
            source: &str,
    ) -> Result<(), DynError> {
        info!("Upgrading to blueprint version {}", &self.metadata.version);
        debug!("Root project dir {:?}", &output_dir);

        for file in self.files() {
            let file = file?;
            let path = file.path();
            let output_path = output_dir.join(file.path_from_template_root());

            if path.is_file() {
                if self.is_excluded(&file.path_from_template_root) {
                    debug!("Copying {:?} to {:?} without templating.", &path, &output_path);
                    fs::copy(path, output_path)?;
                }
                else if output_path.exists() {
                    debug!("Skipping {:?}, file already exists", &path);
                }
                else {
                    debug!("Using template {:?} to render {:?}", &path, &output_path);
                    let contents = fs::read_to_string(&path)?;
                    let contents = engine.render_template(&contents, &values)?;
                    fs::write(output_path, &contents)?;
                }
            }
            else if path.is_dir() {
                if !output_path.is_dir() {
                    debug!("Creating directory {:?}", &output_path);
                    fs::create_dir(&output_path)?;
                }
            }
        }

        debug!("TODO: run upgrade script");

        self.generate_rendr_file(&source, &output_dir, &values)?;

        Ok(())
    }

    fn generate_rendr_file(&self, source: &str, output_dir: &Path, values: &HashMap<&str, &str>) -> Result<(), DynError> {
        debug!("Generating .rendr.yaml file:");
        debug!("  source: {}", source);
        debug!("  output_dir: {}", output_dir.display());
        debug!("  values: {:?}", values);
        let path = output_dir.join(Path::new(".rendr.yaml"));
        let config = RendrConfig::new(source.to_string().clone(), &self.metadata, values);
        let yaml = serde_yaml::to_string(&config)?;
        let mut file = std::fs::File::create(path)?;

        file.write_all(yaml.as_bytes())?;

        Ok(())
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct RendrConfig {
    pub name: String,
    pub version: u32,
    pub author: String,
    pub description: String,
    pub source: String,
    pub values: Vec<RendrConfigValue>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct RendrConfigValue {
    pub name: String,
    pub value: String,
}

impl RendrConfigValue {
    fn new(name: String, value: String) -> Self {
        RendrConfigValue {
            name,
            value,
        }
    }
}

impl RendrConfig {
    fn new(source: String, metadata: &BlueprintMetadata, values: &HashMap<&str, &str>) -> Self {
        let values = values.iter().map(|(k, v)| RendrConfigValue::new(String::from(*k), String::from(*v))).collect();

        RendrConfig {
            name: metadata.name.clone(),
            version: metadata.version,
            author: metadata.author.clone(),
            description: metadata.description.clone(),
            source: source,
            values: values,
        }
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
        for v in self.values.iter() {
            writeln!(f, "- name: {}", v.name);
            writeln!(f, "  value: {}", v.value);
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
                Err(e)    => return Some(Err(e)),
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

    fn path(&self) -> &Path {
        self.dir_entry.path()
    }

    fn path_from_template_root(&self) -> &Path {
        &self.path_from_template_root
    }
}

pub struct Script {
    name: String,
    path: PathBuf,
}

impl Script {
    fn new(name: &str, path: PathBuf) -> Self {
        Script {
            name: name.to_string(),
            path: path,
        }
    }

    fn run(&self, working_dir: &Path, values: &HashMap<&str, &str>) -> Result<(), DynError> {
        info!("Running blueprint script: {}", &self.name);

        #[cfg(debug)]
        debug!("  Blueprint script full path: {:?}", &self.path);
        debug!("  Blueprint script working dir: {:?}", working_dir);

        let output = Command::new("sh")
            .arg("-c")
            .arg(&self.path)
            .envs(values)
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
        ScriptError {
            status,
            msg,
        }
    }
}

impl Error for ScriptError {}

impl Display for ScriptError {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        match self.status {
            Some(status) => write!(f, "Script failed with status {}", status)?,
            None         => write!(f, "Script failed, but didn't exit!")?,
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

#[derive(Deserialize)]
pub struct BlueprintMetadata {
    pub name: String,
    pub version: u32,
    pub author: String,
    pub description: String,
    values: Vec<ValueSpec>,
    #[serde(default)]
    exclusions: Vec<Pattern>,
    #[serde(alias = "git-init")]
    #[serde(default)]
    git_init: bool,
}

#[derive(Deserialize)]
pub struct ValueSpec {
    pub name: String,
    pub description: String,
    pub default: Option<String>,
    #[serde(default)]
    pub required: bool,
}

impl PartialEq for ValueSpec {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

impl Eq for ValueSpec {}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::fs;
    use tempdir::TempDir;
    use crate::blueprint::Blueprint;
    use crate::templating::Mustache;
    use super::*;

    #[test]
    fn parse_example_blueprint_metadata() {
        let blueprint = Blueprint::new("test_assets/example_blueprint").unwrap();

        assert_eq!(blueprint.metadata.name, "example-blueprint");
        assert_eq!(blueprint.metadata.version, 1);
        assert_eq!(blueprint.metadata.author, "Brian S. <brian.stewart@jamf.com>, Tomasz K. <tomasz.kurcz@jamf.com>");
        assert_eq!(blueprint.metadata.description, "Just an example blueprint for `rendr`.");
    }

    #[test]
    fn render_example_blueprint() {
        let blueprint = Blueprint::new("test_assets/example_blueprint").unwrap();

        let output_dir = TempDir::new("my-project").unwrap();

        let mustache = Mustache::new();

        blueprint.render(&mustache, &test_values(), output_dir.path()).unwrap();

        let test = fs::read_to_string(output_dir.path().join("test.yaml")).unwrap();
        let another_test = fs::read_to_string(output_dir.path().join("another-test.yaml")).unwrap();

        assert!(test.find("name: my-project").is_some());
        assert!(test.find("version: 1").is_some());
        assert!(another_test.find("stuff: stuff").is_some());
        assert!(another_test.find("version: 1").is_some());
    }

    #[test]
    fn render_example_blueprint_recursive() {
        let blueprint = Blueprint::new("test_assets/example_blueprint").unwrap();

        let output_dir = TempDir::new("my-project").unwrap();

        let mustache = Mustache::new();

        blueprint.render(&mustache, &test_values(), output_dir.path()).unwrap();

        let test = fs::read_to_string(output_dir.path().join("dir/test.yaml")).unwrap();

        assert!(test.find("name: my-project").is_some());
        assert!(test.find("version: 1").is_some());
    }

    #[test]
    fn exclusions_work() {
        let blueprint = Blueprint::new("test_assets/example_blueprint").unwrap();

        let output_dir = TempDir::new("my-project").unwrap();

        let mustache = Mustache::new();

        blueprint.render(&mustache, &test_values(), output_dir.path()).unwrap();

        let excluded_file = fs::read_to_string(output_dir.path().join("excluded_file")).unwrap();

        assert!(excluded_file.find("{{ name }}").is_some());
    }

    #[test]
    fn glob_exclusions_work() {
        let blueprint = Blueprint::new("test_assets/example_blueprint").unwrap();

        let output_dir = TempDir::new("my-project").unwrap();

        let mustache = Mustache::new();

        blueprint.render(&mustache, &test_values(), output_dir.path()).unwrap();

        let excluded_file1 = fs::read_to_string(output_dir.path().join("excluded_files/foo")).unwrap();
        let excluded_file2 = fs::read_to_string(output_dir.path().join("excluded_files/bar")).unwrap();

        assert!(excluded_file1.find("{{ name }}").is_some());
        assert!(excluded_file2.find("{{ name }}").is_some());
    }

    #[test]
    fn script_can_be_run_successfully() {
        let script = Script::new("some script", PathBuf::from("test_assets/scripts/hello_world.sh"));

        let values = HashMap::new();
        script.run(Path::new("."), &values).unwrap();
    }

    #[test]
    fn running_failing_script_returns_an_error() {
        let script = Script::new("some script", PathBuf::from("test_assets/scripts/failing.sh"));

        let values = HashMap::new();
        if let Ok(()) = script.run(Path::new("."), &values) {
            panic!("The failing script didn't cause an error!");
        }
    }

    #[test]
    fn blueprint_post_script_is_found_and_run() {
        let blueprint = Blueprint::new("test_assets/example_blueprint_with_scripts").unwrap();

        let output_dir = TempDir::new("my-project").unwrap();

        let mustache = Mustache::new();

        blueprint.render(&mustache, &test_values(), output_dir.path()).unwrap();

        let script_output = fs::read_to_string(output_dir.path().join("script_output.md")).unwrap();

        assert_eq!(script_output.as_str(), "something123");
    }

    // Test helpers
    fn test_values() -> HashMap<&'static str, &'static str> {
        vec![("name", "my-project"), ("version", "1"), ("foobar", "stuff")]
            .iter().cloned().collect()
    }
}
