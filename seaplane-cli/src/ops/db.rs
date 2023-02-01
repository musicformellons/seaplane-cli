use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::{
    fs::{FromDisk, ToDisk},
    ops::{flight::Flights, formation::Formations, state_version::StateVersion},
};

/// The in memory "Databases"
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct Db {
    // Where was this "DB" loaded from on disk, so we can persist it back later
    #[serde(skip)]
    loaded_from: Option<PathBuf>,

    #[serde(default)]
    pub state_version: StateVersion,

    /// The in memory Flights database
    #[serde(default)]
    pub flights: Flights,

    /// The in memory Formations database
    #[serde(default)]
    pub formations: Formations,

    /// A *hint* that we should persist at some point. Not gospel
    #[serde(skip)]
    pub needs_persist: bool,
}

impl FromDisk for Db {
    fn set_loaded_from<P: AsRef<Path>>(&mut self, p: P) {
        self.loaded_from = Some(p.as_ref().into());
    }

    fn loaded_from(&self) -> Option<&Path> { self.loaded_from.as_deref() }
}

impl ToDisk for Db {}
