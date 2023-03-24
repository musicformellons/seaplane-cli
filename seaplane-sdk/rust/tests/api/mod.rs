// We have to go through this little bit of indirection because of how integration directory
// structure works.

#[cfg(feature = "compute_api_v2")]
mod compute_v2;
#[cfg(feature = "locks_api_v1")]
mod locks_v1;
#[cfg(feature = "metadata_api_v1")]
mod metadata_v1;
#[cfg(feature = "restrict_api_v1")]
mod restrict_v1;
mod token_v1;

use httpmock::prelude::*;
use once_cell::sync::Lazy;

// To be used with httpmock standalone server for dev testing
// MockServer::connect("127.0.0.1:5000")
// static MOCK_SERVER: Lazy<MockServer> = Lazy::new(|| MockServer::connect("127.0.0.1:5000"));
static MOCK_SERVER: Lazy<MockServer> = Lazy::new(MockServer::start);
