// Copyright ⓒ  2022 Seaplane IO, Inc.
// Licensed under the Apache 2.0 license
// (see LICENSE or <http://opensource.org/licenses/Apache-2.0>) All files in the project carrying such
// notice may not be copied, modified, or distributed except according to those terms.

// only enables the `doc_cfg` feature when the `docsrs` configuration attribute is defined
// `doc_cfg` enables the "This API is only available on feature `FOO`" in the generated docs such as
// on docs.rs
#![cfg_attr(docsrs, feature(doc_cfg))]
#![warn(
    // TODO: we'll get to this
    //missing_docs,
    missing_debug_implementations,
    missing_copy_implementations,
    trivial_casts,
    unused_allocation,
    trivial_numeric_casts
)]
#![forbid(unsafe_code)]

#[cfg(any(
    feature = "compute_api_v1",
    feature = "locks_api_v1",
    feature = "metadata_api_v1",
    feature = "restrict_api_v1"
))]
#[macro_use]
mod macros;
pub mod api;
pub mod base64;
pub mod error;

/// Allows using the exact same traits derived from these dependencies. If re-exported here, that
/// most likely means the derived trait appears in a types public API and you should `use` the
/// re-exported crate trait instead of one from your own dependencies list.
pub mod rexports {
    pub use container_image_ref;
    pub use seaplane_oid;
    pub use strum;
}
