use std::fmt::Display;
use std::fmt::Formatter;
use std::collections::HashMap;
use std::path::Path;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::error::Error;
use std::fs;
use std::io::{self, Write};

use tempdir::TempDir;
use git2::Repository;
use serde::Deserialize;
use serde_yaml;

use crate::templating::TemplatingEngine;

type DynError = Box<dyn Error>;

pub struct Blueprint {
    metadata: BlueprintMetadata,
    dir: BlueprintDir,
    post_script: Option<Script>,
}

impl Blueprint {
    pub fn new(path: &str) -> Result<Blueprint, DynError> {
        let mut blueprint;

        if Path::new(path).exists() {
            blueprint = Self::from_dir(path)?;
        }
        else {
            blueprint = Self::from_repo(path)?;
        }

        blueprint.find_scripts()?;

        Ok(blueprint)
    }

    fn from_repo(path: &str) -> Result<Blueprint, DynError> {
        let dir = TempDir::new("checked_out_blueprint")?;

        Repository::clone(path, &dir)?;

        Self::from_blueprint_dir(BlueprintDir::TempDir(dir))
    }

    fn from_dir(path: &str) -> Result<Blueprint, DynError> {
        let path = Path::new(path).to_path_buf();

        Self::from_blueprint_dir(BlueprintDir::Path(path))
    }

    fn from_blueprint_dir(dir: BlueprintDir) -> Result<Blueprint, DynError> {
        let meta_raw = fs::read_to_string(dir.path().join("metadata.yaml"))?;

        let metadata = serde_yaml::from_str(&meta_raw)?;

        Ok(Blueprint {
            metadata,
            dir,
            post_script: None,
        })
    }

    fn find_scripts(&mut self) -> Result<(), DynError> {
        self.post_script = self.find_script("post-render")?;

        Ok(())
    }

    fn find_script(&mut self, script: &str) -> Result<Option<Script>, DynError> {
        let mut script_path = PathBuf::new();
        script_path.push(self.dir.path().canonicalize()?);
        script_path.push("scripts");
        script_path.push(format!("{}.sh", script));

        if !script_path.exists() {
            #[cfg(debug)]
            eprintln!("No {} script found in blueprint scripts directory - skipping", script);
            return Ok(None);
        }

        Ok(Some(Script::new(script, script_path)))
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

    pub fn render<TE: TemplatingEngine>(
            &self,
            engine: &TE,
            values: &HashMap<&str, &str>,
            output_dir: &Path,
    ) -> Result<(), DynError> {
        // Create our output directory if it doesn't exist yet.
        if !output_dir.is_dir() {
            fs::create_dir(output_dir)?;
        }

        self.render_rec(engine, values, &self.dir.path().join("blueprint"), output_dir)?;

        if let Some(post_script) = &self.post_script {
            post_script.run(output_dir, values)?;
        }

        Ok(())
    }

    pub fn render_rec<TE: TemplatingEngine>(
            &self,
            engine: &TE,
            values: &HashMap<&str, &str>,
            src_dir: &Path,
            output_dir: &Path,
    ) -> Result<(), DynError> {
        // Iterate through the blueprint templates and render them into our output
        // directory.

        println!("render_rec: {:?} {:?}", src_dir, output_dir);
        for entry in fs::read_dir(src_dir)? {
            let path = entry?.path();

            if path.is_file() {
                println!("Found file {:?}", &path);

                let filename = path.file_name()
                    .unwrap()
                    .to_str()
                    .expect("Invalid utf8 in filepath.");

                let contents = fs::read_to_string(&path)?;

                let contents = engine.render_template(&contents, &values)?;

                fs::write(output_dir.join(filename), &contents)?;
            }
            else if path.is_dir() {
                let dirname = path.file_name()
                    .unwrap()
                    .to_str()
                    .expect("Invalid utf8 in filepath.");
                let output_dir = output_dir.join(dirname);
                if !output_dir.is_dir() {
                    fs::create_dir(&output_dir)?;
                }
                self.render_rec(engine, values, &path, &output_dir)?;
            }
        }

        Ok(())
    }
}

struct Script {
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
        println!("Running blueprint script: {}", &self.name);

        #[cfg(debug)]
        println!("  Blueprint script full path: {:?}", &self.path);
        println!("  Blueprint script working dir: {:?}", working_dir);

        let output = Command::new("sh")
            .arg("-c")
            .arg(&self.path)
            .envs(values)
            .current_dir(working_dir)
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .output()
            .expect("failed to execute script");

        println!("Status: {}", output.status);

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

enum BlueprintDir {
    TempDir(TempDir),
    Path(PathBuf),
}

impl BlueprintDir {
    fn path(&self) -> &Path {
        use BlueprintDir::*;

        match self {
            TempDir(tmpdir) => tmpdir.path(),
            Path(path)      => &path,
        }
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
struct BlueprintMetadata {
    name: String,
    version: u32,
    author: String,
    description: String,
    values: Vec<ValueSpec>,
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
        assert_eq!(blueprint.metadata.description, "Just an example blueprint for `express`.");
    }

    #[test]
    fn render_example_blueprint() {
        let blueprint = Blueprint::new("test_assets/example_blueprint").unwrap();

        let output_dir = TempDir::new("my-project").unwrap();

        let values: HashMap<_, _> = vec![("name", "my-project"), ("version", "1"), ("foobar", "stuff")]
            .iter().cloned().collect();

        let mustache = Mustache::new();

        blueprint.render(&mustache, &values, output_dir.path()).unwrap();

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

        let values: HashMap<_, _> = vec![("name", "my-project"), ("version", "1"), ("foobar", "stuff")]
            .iter().cloned().collect();

        let mustache = Mustache::new();

        blueprint.render(&mustache, &values, output_dir.path()).unwrap();

        let test = fs::read_to_string(output_dir.path().join("dir/test.yaml")).unwrap();

        assert!(test.find("name: my-project").is_some());
        assert!(test.find("version: 1").is_some());
    }

    #[test]
    fn script_can_be_run_successfully() {
        let script = Script::new("some script", PathBuf::from("test_assets/scripts/hello_world.sh"));

        let values = HashMap::new();
        script.run(Path::new("."), &values).unwrap();
    }

    #[test]
    fn run_script_returns_error_on_failing_script() {
        let script = Script::new("some script", PathBuf::from("test_assets/scripts/failing.sh"));

        let values = HashMap::new();
        if let Ok(()) = script.run(Path::new("."), &values) {
            panic!("The failing script didn't cause an error!");
        }
    }
}
