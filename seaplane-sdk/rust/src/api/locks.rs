pub mod error;
#[cfg(feature = "locks_api_v1")]
pub mod v1;

/// The base URL for our Locks API endpoints
pub static LOCKS_API_URL: &str = "https://metadata.cplane.cloud/";
