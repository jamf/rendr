use std::path::{Path, PathBuf};

use git2::RemoteCallbacks;
use tempdir::TempDir;
use git2::Repository;
use thiserror::Error;

// type DynError = Box<dyn Error>;

pub enum Source {
    Local(PathBuf),
    Remote(RemoteSource),
}

impl Source {
    pub fn new(source: &str, callbacks: Option<RemoteCallbacks>) -> Result<Self, BlueprintSourceError> {
        let path = Path::new(source);

        if path.exists() {
            return Ok(Self::local(path)?);
        }

        Self::remote(source, callbacks)
    }

    fn local(path: impl AsRef<Path>) -> Result<Self, BlueprintSourceError> {
        Ok(Source::Local(
            path.as_ref()
                .canonicalize()
                .map_err(|e| BlueprintSourceError::LocalReadError(e))?
        ))
    }

    fn remote(url: &str, callbacks: Option<RemoteCallbacks>) -> Result<Self, BlueprintSourceError> {
        let dir = TempDir::new("checked_out_blueprint")
            .map_err(|e| BlueprintSourceError::TempDirCreationError(e))?;

        // Prepare fetch options.
        let mut fo = git2::FetchOptions::new();

        // Set callbacks if provided.
        if let Some(callbacks) = callbacks {
            fo.remote_callbacks(callbacks);
        }

        // Prepare builder.
        let mut builder = git2::build::RepoBuilder::new();
        builder.fetch_options(fo);

        // Clone the project.
        builder.clone(url, dir.as_ref())?;

        Ok(Source::Remote(RemoteSource {
            url: url.to_string(),
            checked_out: dir,
        }))
    }

    /// The local path where the blueprint data can be found for parsing.
    pub fn path(&self) -> &Path {
        use Source::*;

        match self {
            Remote(tmpdir) => tmpdir.path(),
            Local(path) => &path,
        }
    }

    /// A blueprint locator to be used in the .rendr.yaml file
    /// in the rendered project.
    pub fn to_string(&self, from: impl AsRef<Path>) -> String {
        use Source::*;

        let from = from.as_ref().canonicalize().unwrap();

        match self {
            Local(path) => pathdiff::diff_paths(path, from)
                .unwrap()
                .into_os_string()
                .into_string()
                .unwrap(),
            Remote(src) => src.url().to_string(),
        }
    }
}

pub struct RemoteSource {
    url: String,
    checked_out: TempDir,
}

impl RemoteSource {
    fn path(&self) -> &Path {
        self.checked_out.path()
    }

    fn url(&self) -> &str {
        &self.url
    }
}

#[derive(Error, Debug)]
pub enum BlueprintSourceError {
    #[error("failed to read the local blueprint")]
    LocalReadError(#[source] std::io::Error),

    #[error("failed to create a temporary directory")]
    TempDirCreationError(#[source] std::io::Error),

    #[error("failed to clone the git repository")]
    RepoCloneError(#[from] git2::Error),
}

#[test]
fn source_canonicalizes_its_path_on_init() {
    let source = Source::new("test_assets", None).unwrap();

    assert!(source.path().is_absolute());
}

#[test]
fn source_calculates_relative_path_correctly() {
    let source = Source::new("test_assets/example_blueprint", None).unwrap();
    let project_dir = ".";

    assert_eq!(
        source.to_string(project_dir),
        "test_assets/example_blueprint"
    );
}
