use thiserror::Error as ThisError;

#[allow(missing_copy_implementations)]
#[derive(ThisError, Debug, Clone, PartialEq, Eq)]
pub enum MetadataError {
    #[error("request did not include the required key")]
    MissingMetadataKey,
    #[error("request must target either key or range")]
    IncorrectMetadataRequestTarget,
}
