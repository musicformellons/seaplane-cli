use std::{
    fs, io,
    path::{Path, PathBuf},
};

use directories::ProjectDirs;
use serde::{de::DeserializeOwned, Serialize};
use tempfile::NamedTempFile;

use crate::{
    cli::{CliCommand, SeaplaneInit},
    context::Ctx,
    error::{CliError, CliErrorKind, Context, Result},
    printer::Color,
};

/// A utility function to get the correct "project" directories in a platform specific manner
#[inline]
fn project_dirs() -> Option<ProjectDirs> {
    directories::ProjectDirs::from("io", "Seaplane", "seaplane")
}

/// Finds all appropriate configuration directories in a platform specific manner
pub fn conf_dirs() -> Vec<PathBuf> {
    let mut dirs = Vec::new();
    if let Some(proj_dirs) = project_dirs() {
        dirs.push(proj_dirs.config_dir().to_owned());
    }
    if let Some(base_dirs) = directories::BaseDirs::new() {
        // On Linux ProjectDirs already adds ~/.config/seaplane, but not on macOS or Windows
        if !cfg!(target_os = "linux") {
            dirs.push(base_dirs.home_dir().join(".config/seaplane"));
        }
        dirs.push(base_dirs.home_dir().join(".seaplane"));
    }
    dirs
}

/// A utility function to get the correct data directory
#[cfg(not(feature = "ui_tests"))]
#[inline]
pub fn data_dir() -> PathBuf {
    project_dirs()
        .expect("Failed to determine usable directories")
        .data_dir()
        .to_owned()
}

#[cfg(feature = "ui_tests")]
#[cfg_attr(feature = "ui_tests", inline)]
pub fn data_dir() -> PathBuf { std::env::current_dir().unwrap() }

/// A struct that writes to a tempfile and persists to a given location atomically on Drop
#[derive(Debug)]
pub struct AtomicFile<'p> {
    path: &'p Path,
    temp_file: Option<NamedTempFile>,
}

impl<'p> AtomicFile<'p> {
    /// Creates a new temporary file that will eventually be persisted to path `p`
    pub fn new(p: &'p Path) -> Result<Self> {
        Ok(Self { path: p, temp_file: Some(NamedTempFile::new()?) })
    }

    /// Gives a chance to persist the file and retrieve the error if any
    #[allow(dead_code)]
    pub fn persist(mut self) -> Result<()> {
        let tf = self.temp_file.take().unwrap();
        tf.persist(self.path).map(|_| ()).map_err(CliError::from)
    }

    /// Returns the `Path` of the underlying temporary file
    pub fn temp_path(&self) -> &Path { self.temp_file.as_ref().unwrap().path() }
}

impl<'p> io::Write for AtomicFile<'p> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        if let Some(ref mut tf) = &mut self.temp_file {
            return tf.write(buf);
        }

        Ok(0)
    }

    fn flush(&mut self) -> io::Result<()> {
        if let Some(ref mut tf) = &mut self.temp_file {
            return tf.flush();
        }

        Ok(())
    }
}

impl<'p> Drop for AtomicFile<'p> {
    fn drop(&mut self) {
        // Swallow the error
        let tf = self.temp_file.take().unwrap();
        let _ = tf.persist(self.path);
    }
}

// TODO: make the deserializer generic
pub trait FromDisk {
    /// Allows one to save or deserialize what path the item was loaded from
    fn set_loaded_from<P: AsRef<Path>>(&mut self, _p: P) {}

    /// If saved, get the path the item was loaded from
    fn loaded_from(&self) -> Option<&Path> { None }

    /// Deserialize from some given path
    fn load<P: AsRef<Path>>(p: P) -> Result<Self>
    where
        Self: Sized + DeserializeOwned,
    {
        let path = p.as_ref();

        let json_str = match fs::read_to_string(path) {
            Ok(s) => s,
            Err(e) => {
                // If it's a file missing error we try to auto-initialize, then return the error if
                // it happens again
                if e.kind() == io::ErrorKind::NotFound {
                    let mut ctx = Ctx::default();
                    ctx.internal_run = true;
                    SeaplaneInit.run(&mut ctx)?;

                    fs::read_to_string(path)
                        .map_err(CliError::from)
                        .context("\n\tpath: ")
                        .with_color_context(|| (Color::Yellow, format!("{path:?}\n")))?
                } else {
                    return Err(CliError::from(e));
                }
            }
        };
        let mut item: Self = serde_json::from_str(&json_str)
            .map_err(CliError::from)
            .context("\n\tpath: ")
            .with_color_context(|| (Color::Yellow, format!("{path:?}\n")))?;

        item.set_loaded_from(p);

        Ok(item)
    }
}

// TODO: make the serializer generic
pub trait ToDisk: FromDisk {
    /// Serializes itself to the given path
    fn persist(&self) -> Result<()>
    where
        Self: Sized + Serialize,
    {
        if let Some(path) = self.loaded_from() {
            let file = AtomicFile::new(path)?;
            // TODO: long term consider something like SQLite
            Ok(serde_json::to_writer(file, self)
                .map_err(CliError::from)
                .context("\n\tpath: ")
                .with_color_context(|| (Color::Yellow, format!("{path:?}")))?)
        } else {
            Err(CliErrorKind::MissingPath.into_err())
        }
    }
}
