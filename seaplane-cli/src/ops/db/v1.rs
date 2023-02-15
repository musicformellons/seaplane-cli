//! Legacy Db Models used only in upgrading from old local state

use std::{
    fmt, fs,
    path::{Path},
};

use rand::Rng;
use serde::{Deserialize, Serialize};

use crate::{
    error::Result,
    fs::FromDisk,
    ops::{flight::v1::Flights, formation::v1::Formations, state_version::StateVersion},
};

/// A DB capable of loading v0 and v1 state
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct Dbv1 {
    #[serde(default)]
    pub state_version: StateVersion,

    /// The in memory Formations database
    #[serde(default)]
    pub formations: Formations,

    /// The in memory Formations database
    #[serde(default)]
    pub flights: Flights,

    /// A *hint* that we should persist at some point. Not gospel
    #[serde(skip)]
    pub needs_persist: bool,
}

impl Dbv1 {
    pub fn load_v0<P: AsRef<Path>>(flights: P, formations: P) -> Result<Self> {
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

    pub fn load_v1<P: AsRef<Path> + Clone>(state: P) -> Result<Self> {
        let ret = Self::load(state.clone())?;
        fs::remove_file(state)?;
        Ok(ret)
    }
}

impl FromDisk for Dbv1 {}

#[derive(Deserialize, Serialize, Copy, Clone, Hash, PartialEq, Eq)]
#[serde(transparent)]
pub struct Id {
    #[serde(
        serialize_with = "hex::serde::serialize",
        deserialize_with = "hex::serde::deserialize"
    )]
    pub inner: [u8; 32],
}

impl Default for Id {
    fn default() -> Self { Self { inner: rand::thread_rng().gen() } }
}

impl Id {
    pub fn new() -> Self { Self::default() }
}

impl fmt::Display for Id {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", hex::encode(self.inner))
    }
}

impl fmt::Debug for Id {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { write!(f, "Id [ {self} ]") }
}
