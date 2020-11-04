use std::error::Error;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

use git2::{Cred, RemoteCallbacks};
use log::{debug, error};
use tempdir::TempDir;
use text_io::read;
use thiserror::Error;

use super::BlueprintAuth;

type DynError = Box<dyn Error>;

pub enum Source {
    Local(PathBuf),
    Remote(RemoteSource),
}

impl Source {
    pub fn new(source: &str, auth: Option<BlueprintAuth>) -> Result<Self, BlueprintSourceError> {
        let path = Path::new(source);
        debug!("Initializing blueprint source from {}", path.display());

        if path.exists() {
            debug!("Source path exists, loading");
            return Ok(Self::local(path)?);
        }

        debug!("Source path does not exist, loading from remote source");
        let callbacks = Source::prepare_callbacks(auth);
        Self::remote(source, Some(callbacks))
    }

    fn local(path: impl AsRef<Path>) -> Result<Self, BlueprintSourceError> {
        Ok(Source::Local(
            path.as_ref()
                .canonicalize()
                .map_err(|e| BlueprintSourceError::LocalReadError(e))?,
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

    pub fn prepare_callbacks<'c>(auth: Option<BlueprintAuth>) -> RemoteCallbacks<'c> {
        if auth.is_none() {
            return RemoteCallbacks::new();
        }

        let auth = auth.unwrap();

        let provided_user = auth.user;
        let provided_pass = auth.password;
        let provided_ssh_key = auth.ssh_key;

        let mut callbacks = RemoteCallbacks::new();
        let mut auth_retries = 3;

        callbacks.credentials(move |_url, username_from_url, allowed_types| {
            debug!("Git requested cred types: {:?}", allowed_types);

            if auth_retries < 1 {
                panic!("exceeded 3 auth retries; invalid credentials?");
            }

            if allowed_types.is_ssh_key() {
                auth_retries -= 1;

                if let Some(ssh_key) = &provided_ssh_key {
                    let path = Path::new(ssh_key.as_str());
                    return Cred::ssh_key(
                        &get_username(&provided_user, username_from_url).unwrap(),
                        None,
                        path,
                        None,
                    );
                } else {
                    return Cred::ssh_key(
                        &get_username(&provided_user, username_from_url).unwrap(),
                        None,
                        &Path::new(&format!("{}/.ssh/id_rsa", std::env::var("HOME").unwrap())),
                        None,
                    );
                }
            } else if allowed_types.is_username() {
                return Cred::username(&get_username(&provided_user, username_from_url).unwrap());
            } else if allowed_types.is_user_pass_plaintext() {
                auth_retries -= 1;

                return Cred::userpass_plaintext(
                    &get_username(&provided_user, username_from_url).unwrap(),
                    &get_password(&provided_pass).unwrap(),
                );
            }

            panic!(
                "git requested an unimplemented credential type: {:?}",
                allowed_types
            )
        });

        fn get_username(
            provided_user: &Option<String>,
            username_from_url: Option<&str>,
        ) -> Result<String, DynError> {
            if let Some(username) = provided_user {
                return Ok(username.to_string());
            }

            if let Some(username) = username_from_url {
                return Ok(username.to_string());
            }

            print!("Username: ");
            io::stdout().flush().unwrap();
            Ok(read!("{}\n"))
        }

        fn get_password(provided_pass: &Option<String>) -> Result<String, DynError> {
            if let Some(pass) = provided_pass {
                return Ok(pass.to_string());
            }

            Ok(rpassword::read_password_from_tty(Some("Password: "))?)
        }

        callbacks
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
