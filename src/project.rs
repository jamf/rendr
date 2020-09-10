use std::path::{Path, PathBuf};
use std::error::Error;
use std::fs;

use crate::blueprint::{Values, RendrConfig, Blueprint};

type DynError = Box<dyn Error>;

pub struct Project<'p> {
    path: &'p Path,
    meta: RendrConfig,
}

impl<'p> Project<'p> {
    pub fn new(path: &'p impl AsRef<Path>) -> Result<Self, DynError> {
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

    pub fn blueprint(&self) -> Result<Blueprint, DynError> {
        Blueprint::new(
            self.path
                .join(&self.meta.source)
                .as_os_str()
                .to_str()
                .unwrap()
        )
    }
}