use thiserror::Error as ThisError;

#[allow(missing_copy_implementations)]
#[derive(ThisError, Debug, Clone, PartialEq, Eq)]
pub enum LocksError {
    #[error("locks requests must target either a lock by name or a held lock")]
    IncorrectLocksRequestTarget,
}
