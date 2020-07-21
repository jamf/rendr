use std::error::Error;
use std::path::Path;
use std::path::PathBuf;

use tempdir::TempDir;
use git2::Repository;

type DynError = Box<dyn Error>;

pub enum Source {
    Local(PathBuf),
    Remote(RemoteSource),
}

impl Source {
    pub fn new(source: &str) -> Result<Self, DynError> {
        let path = Path::new(source);

        if path.exists() {
            return Ok(Self::local(path)?);
        }
        Ok(Self::remote(source)?)
    }

    fn local(path: impl AsRef<Path>) -> Result<Self, DynError> {
        Ok(Source::Local(path.as_ref().canonicalize()?))
    }

    fn remote(url: &str) -> Result<Self, DynError> {
        let dir = TempDir::new("checked_out_blueprint")?;

        Repository::clone(url, &dir)?;

        Ok(Source::Remote(RemoteSource {
            url: url.to_string(),
            checked_out: dir,
        }))
    }

    pub fn path(&self) -> &Path {
        use Source::*;

        match self {
            Remote(tmpdir) => tmpdir.path(),
            Local(path)    => &path,
        }
    }
}

struct RemoteSource {
    url: String,
    checked_out: TempDir,
}

impl RemoteSource {
    fn path(&self) -> &Path {
        self.checked_out.path()
    }
}