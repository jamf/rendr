use std::path::PathBuf;
use std::fmt::Display;
use std::fmt::Formatter;
use std::collections::HashMap;
use std::path::Path;
use std::error::Error;
use std::fs;

use tempdir::TempDir;
use git2::Repository;
use serde::Deserialize;
use serde_yaml;

use crate::templating::TemplatingEngine;

type DynError = Box<dyn Error>;

pub struct Blueprint {
    metadata: BlueprintMetadata,
    dir: BlueprintDir,
}

impl Blueprint {
    pub fn from_repo(path: &str) -> Result<Blueprint, DynError> {
        let dir = TempDir::new("checked_out_blueprint")?;

        Repository::clone(path, &dir)?;

        Self::new(BlueprintDir::TempDir(dir))
    }

    pub fn from_dir(path: &str) -> Result<Blueprint, DynError> {
        let path = Path::new(path).to_path_buf();

        Self::new(BlueprintDir::Path(path))
    }

    fn new(dir: BlueprintDir) -> Result<Blueprint, DynError> {
        let meta_raw = fs::read_to_string(dir.path().join("metadata.yaml"))?;

        let metadata = serde_yaml::from_str(&meta_raw)?;

        Ok(Blueprint {
            metadata,
            dir,
        })
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
    values: Vec<Value>,
}

#[derive(Deserialize)]
struct Value {
    name: String,
    description: String,
    default: Option<String>,
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::fs;
    use tempdir::TempDir;
    use crate::blueprint::Blueprint;
    use crate::templating::Mustache;
    
    #[test]
    fn parse_example_blueprint_metadata() {
        let blueprint = Blueprint::from_dir("test_assets/example_blueprint").unwrap();

        assert_eq!(blueprint.metadata.name, "example-blueprint");
        assert_eq!(blueprint.metadata.version, 1);
        assert_eq!(blueprint.metadata.author, "Brian S. <brian.stewart@jamf.com>, Tomasz K. <tomasz.kurcz@jamf.com>");
        assert_eq!(blueprint.metadata.description, "Just an example blueprint for `express`.");
    }

    #[test]
    fn render_example_blueprint() {
        let blueprint = Blueprint::from_dir("test_assets/example_blueprint").unwrap();

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
        let blueprint = Blueprint::from_dir("test_assets/example_blueprint").unwrap();

        let output_dir = TempDir::new("my-project").unwrap();

        let values: HashMap<_, _> = vec![("name", "my-project"), ("version", "1"), ("foobar", "stuff")]
            .iter().cloned().collect();
        
        let mustache = Mustache::new();

        blueprint.render(&mustache, &values, output_dir.path()).unwrap();

        let test = fs::read_to_string(output_dir.path().join("dir/test.yaml")).unwrap();

        assert!(test.find("name: my-project").is_some());
        assert!(test.find("version: 1").is_some());
    }
}
