use std::error::Error;
use std::path::{Path, PathBuf};
use std::io::{self, Write};

use tempdir::TempDir;
use text_io::read;
use git2::{Cred, RemoteCallbacks};
use log::debug;

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

        // Prepare callbacks.
        let mut auth_retries = 3;
        let mut callbacks = RemoteCallbacks::new();
        callbacks.credentials(|_url, username_from_url, allowed_types| {
            debug!("Git requested cred types: {:?}", allowed_types);

            if auth_retries < 1 {
                panic!("exceeded 3 auth retries; invalid credentials?");
            }

            if allowed_types.is_ssh_key() {
                auth_retries -= 1;

                return Cred::ssh_key(
                    &get_username(username_from_url).unwrap(),
                    None,
                    Path::new(&format!("{}/.ssh/id_rsa", std::env::var("HOME").unwrap())),
                    None,
                );
            } else if allowed_types.is_username() {
                return Cred::username(
                    &get_username(username_from_url).unwrap()
                );
            } else if allowed_types.is_user_pass_plaintext() {
                auth_retries -= 1;

                return Cred::userpass_plaintext(
                    &get_username(username_from_url).unwrap(),
                    &get_password().unwrap(),
                );
            }

            panic!("git requested an unimplemented credential type: {:?}", allowed_types)
        });

        fn get_username(username_from_url: Option<&str>) -> Result<String, DynError> {
            if let Some(username) = username_from_url {
                return Ok(username.to_string());
            }

            print!("Username: ");
            io::stdout().flush().unwrap();
            Ok(read!("{}\n"))
        }

        fn get_password() -> Result<String, DynError> {
            Ok(rpassword::read_password_from_tty(Some("Password: "))?)
        }

        // Prepare fetch options.
        let mut fo = git2::FetchOptions::new();
        fo.remote_callbacks(callbacks);

        // Prepare builder.
        let mut builder = git2::build::RepoBuilder::new();
        builder.fetch_options(fo);

        // Clone the project.
        builder.clone(url, dir.as_ref())?;

        // Repository::clone(url, &dir)?;

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
            Local(path)    => &path,
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

#[test]
fn source_canonicalizes_its_path_on_init() {
    let source = Source::new("test_assets").unwrap();

    assert!(source.path().is_absolute());
}

#[test]
fn source_calculates_relative_path_correctly() {
    let source = Source::new("test_assets/example_blueprint").unwrap();
    let project_dir = ".";

    assert_eq!(source.to_string(project_dir), "test_assets/example_blueprint");
}