use std::collections::HashMap;
use std::path::Path;
use std::error::Error;
use std::fs;

use tempdir::TempDir;
use git2::Repository;

use crate::templating::TemplatingEngine;

type DynError = Box<dyn Error>;

pub struct Blueprint {
    dir: TempDir,
}

impl Blueprint {
    pub fn from_remote_repo(url: &str) -> Result<Blueprint, DynError> {
        let dir = TempDir::new("checked_out_blueprint")?;

        Repository::clone(url, &dir)?;

        Ok(Blueprint {
            dir,
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