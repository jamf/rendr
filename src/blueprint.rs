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

impl Blueprint {
    pub fn from_repo_location(path: &str) -> Result<Blueprint, DynError> {
        let dir = TempDir::new("checked_out_blueprint")?;

        Repository::clone(path, &dir)?;

        let meta_raw = fs::read_to_string(dir.path().join("metadata.yaml"))?;

        let metadata = serde_yaml::from_str(&meta_raw)?;

        Ok(Blueprint {
            metadata,
            dir: BlueprintDir::TempDir(dir),
        })
    }

    pub fn from_dir(path: &str) -> Result<Blueprint, DynError> {
        let path = Path::new(path);

        let meta_raw = fs::read_to_string(path.join("metadata.yaml"))?;

        let metadata = serde_yaml::from_str(&meta_raw)?;

        Ok(Blueprint {
            metadata,
            dir: BlueprintDir::Path(path.to_path_buf()),
        })
    }

    pub fn render<TE: TemplatingEngine>(
            &self,
            engine: &TE,
            values: &HashMap<&str, &str>,
            output_dir: &Path,
    ) -> Result<(), DynError> {
        // Create our output directory.
        fs::create_dir(output_dir)?;

        // Iterate through the blueprint templates and render them into our output
        // directory.  
        if output_dir.is_dir() {
            for entry in fs::read_dir(&self.dir.path().join("blueprint"))? {
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
            }
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
    }
}
