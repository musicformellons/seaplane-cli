use std::fmt;

use base64::{
    alphabet::URL_SAFE,
    engine::{general_purpose::NO_PAD, Engine, GeneralPurpose},
};
use reqwest::Url;
use serde::{Deserialize, Serialize};

/// Convenience macro for implementing encode/decode getters/setters for a struct with an inner
/// `Base64Encoded`
// TODO: Is this actually any more "hidden" than a trait? I'm not sure
#[macro_export]
macro_rules! impl_base64 {
    ($a:ty) => {
        impl $a {
            /// Constructs from an unencoded byte array, encoding with URL-safe base64 in the
            /// process
            pub fn from_unencoded(unencoded: impl AsRef<[u8]>) -> Self {
                Self { inner: Base64Encoded::from_unencoded(unencoded) }
            }

            /// Constructs a `Base64Encoded`, assuming the input is already encoded.
            pub fn from_encoded(encoded: impl Into<String>) -> Self {
                Self { inner: Base64Encoded::from_encoded(encoded) }
            }

            /// Returns the inner string
            pub fn encoded(&self) -> &str { self.inner.encoded() }

            /// Returns the result of decoding the inner string.
            /// # Panics
            /// Will panic if the inner string is not correctly encoded.
            pub fn decode(&self) -> Vec<u8> { self.inner.decoded() }
        }
    };
}

/// Holds a [URL-safe base64 encoded](https://datatracker.ietf.org/doc/html/rfc4648#section-5) string
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub(crate) struct Base64Encoded(String);

impl fmt::Display for Base64Encoded {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { write!(f, "{}", self.0) }
}

impl AsRef<str> for Base64Encoded {
    fn as_ref(&self) -> &str { &self.0 }
}

impl Base64Encoded {
    /// Constructs from an unencoded byte array, encoding with URL-safe base64 in the process
    #[allow(dead_code)]
    pub(crate) fn from_unencoded(unencoded: impl AsRef<[u8]>) -> Self {
        let engine = GeneralPurpose::new(&URL_SAFE, NO_PAD);
        Base64Encoded(engine.encode(unencoded))
    }

    /// Constructs a `Base64Encoded`, assuming the input is already encoded.
    // This is a reasonable thing to provide, as the majority of the time this function will be used
    // with the keys that are returned by the API, which are already encoded
    #[allow(dead_code)]
    pub(crate) fn from_encoded(encoded: impl Into<String>) -> Self { Base64Encoded(encoded.into()) }

    /// Returns the result of decoding the inner string.
    /// # Panics
    /// Will panic if the inner string is not correctly encoded.
    #[allow(dead_code)]
    pub(crate) fn decoded(&self) -> Vec<u8> {
        let engine = GeneralPurpose::new(&URL_SAFE, NO_PAD);
        engine
            .decode(&self.0)
            .expect("failed to decode, should be safe by construction")
    }

    /// Returns the inner string
    #[allow(dead_code)]
    pub(crate) fn encoded(&self) -> &str { &self.0 }
}

/// Adds a path segment to endpoint_url in the form "base64:{value}", assumes the path ends in /
// Needed as Url::join parses the new ending as a URL, and thus treats "base64" as a scheme.
// There might be a good reason it parses it though
pub fn add_base64_path_segment<S: AsRef<str>>(mut url: Url, value: S) -> Url {
    let new_path = format!("{}base64:{}", url.path(), value.as_ref());
    url.set_path(&new_path);
    url
}
