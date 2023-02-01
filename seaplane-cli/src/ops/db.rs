use std::{
    fs,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};

use crate::{
    context::Ctx,
    error::Result,
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

impl Db {
    /// Attempt to load state starting at v0 and upgrade until the current state version. State
    /// will be persisted at the current version.
    pub fn load_and_upgrade(ctx: &Ctx) -> Result<Self> {
        // optimistically load v1 (current)
        let mut v1 = Self::load_v1(ctx.state_file())?;

        // attempt to load v0
        let v0 = Self::load_v0(ctx.flights_file(), ctx.formations_file())?;
        // needs_persist will be true from in load_v0 if we successfully loaded either some flights
        // or formations from the v0 state.
        if v0.needs_persist {
            // Extend (and which does the deduplication) our current state from the old state
            v1.formations.extend(&v0.formations);
            v1.flights.extend(&v0.flights);
        }

        // Save current state
        v1.persist()?;

        Ok(v1)
    }

    fn load_v0<P: AsRef<Path>>(flights: P, formations: P) -> Result<Self> {
        let flights = flights.as_ref();
        let formations = formations.as_ref();
        if flights.exists() || formations.exists() {
            let ret = Self {
                flights: FromDisk::load(flights).unwrap_or_default(),
                formations: FromDisk::load(formations).unwrap_or_default(),
                needs_persist: true,
                ..Self::default()
            };
            fs::remove_file(flights)?;
            fs::remove_file(formations)?;
            return Ok(ret);
        }
        Ok(Self::default())
    }

    fn load_v1<P: AsRef<Path>>(state: P) -> Result<Self> { Self::load(state) }
}

impl FromDisk for Db {
    fn set_loaded_from<P: AsRef<Path>>(&mut self, p: P) {
        self.loaded_from = Some(p.as_ref().into());
    }

    fn loaded_from(&self) -> Option<&Path> { self.loaded_from.as_deref() }
}

impl ToDisk for Db {}
