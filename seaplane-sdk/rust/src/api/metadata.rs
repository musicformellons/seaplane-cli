pub mod error;
#[cfg(feature = "metadata_api_v1")]
pub mod v1;

/// The base URL for our Metadata endpoints
pub static METADATA_API_URL: &str = "https://metadata.cplane.cloud/";
