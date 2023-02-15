use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::{
    context::Ctx,
    error::Result,
    fs::{FromDisk, ToDisk},
    ops::{formation::Formations, state_version::StateVersion},
};

/// The in memory "Databases"
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct Db {
    // Where was this "DB" loaded from on disk, so we can persist it back later
    #[serde(skip)]
    loaded_from: Option<PathBuf>,

    #[serde(default)]
    pub state_version: StateVersion,

    /// The in memory Formations database
    #[serde(default)]
    pub formations: Formations,

    /// A *hint* that we should persist at some point. Not gospel
    #[serde(skip)]
    pub needs_persist: bool,
}

impl Db {
    /// Try to load the earliest state possible and upgrade to current.
    /// State will be persisted at the current version.
    pub fn load_and_upgrade(ctx: &Ctx) -> Result<Self> {
        // optimistically load v2 (current)
        let current = Self::load(ctx.state_file())?;

        // Save current state
        current.persist()?;

        Ok(current)
    }
}

impl FromDisk for Db {
    fn set_loaded_from<P: AsRef<Path>>(&mut self, p: P) {
        self.loaded_from = Some(p.as_ref().into());
    }

    fn loaded_from(&self) -> Option<&Path> { self.loaded_from.as_deref() }
}

impl ToDisk for Db {}
