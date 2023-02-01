use serde::{Deserialize, Serialize};

pub const CURRENT_STATE_VERSION: StateVersion = StateVersion { major: 1, minor: 0 };

#[derive(Copy, Clone, Default, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct StateVersion {
    #[serde(default)]
    pub major: usize,
    #[serde(default)]
    pub minor: usize,
}
