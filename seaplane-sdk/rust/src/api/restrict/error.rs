use thiserror::Error as ThisError;

#[allow(missing_copy_implementations)]
#[derive(ThisError, Debug, Clone, PartialEq, Eq)]
pub enum RestrictError {
    #[error("restrict requests must target all restrictions, an api, or an api and a key")]
    IncorrectRestrictRequestTarget,
    #[error("the requirements specified in the builder are in conflict and invalid")]
    ConflictingRequirements,
}
