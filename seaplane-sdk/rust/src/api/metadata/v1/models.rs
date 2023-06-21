use std::fmt;

use serde::{Deserialize, Serialize};

use crate::{api::shared::v1::RangeQueryContext, base64::Base64Encoded, impl_base64};

/// A single key value pair, encoded in url-safe base64.
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub struct KeyValue {
    pub key: Key,
    pub value: Value,
}

impl KeyValue {
    pub fn into_value(self) -> Value { self.value }
}

/// A single key with which to access a value in the store, encoded in url-safe base64.
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
#[serde(transparent)]
pub struct Key {
    inner: Base64Encoded,
}
impl_base64!(Key);

impl AsRef<str> for Key {
    fn as_ref(&self) -> &str { self.inner.as_ref() }
}

impl fmt::Display for Key {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { write!(f, "{}", self.inner) }
}

/// The raw bytes stored at a given key, encoded in url-safe base64.
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
#[serde(transparent)]
pub struct Value {
    inner: Base64Encoded,
}
impl_base64!(Value);

impl AsRef<str> for Value {
    fn as_ref(&self) -> &str { self.inner.as_ref() }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { write!(f, "{}", self.inner) }
}

/// The target of a request, representing either a single key or a range of keys.
#[derive(Debug, PartialEq, Eq)]
pub enum RequestTarget {
    Key(Key),
    Range(RangeQueryContext<Key>),
}

/// The response given from a range query
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct KeyValueRange {
    /// A lower bound of the next page of results
    pub next_key: Option<Key>,
    /// The range of key value pairs returned
    pub kvs: Vec<KeyValue>,
}

#[cfg(test)]
mod metadata_models_test {
    use super::*;

    #[test]
    fn key_value_range_deserialize() {
        let deserialized = serde_json::from_str(
            "{\"next_key\":\"bmV4dCBrZX\",\"kvs\":[{\"key\":\"aGVsbG8\",\"value\":\"dmFsdWU\"}]}",
        )
        .unwrap();

        assert_eq!(
            KeyValueRange {
                next_key: Some(Key::from_encoded("bmV4dCBrZX")),
                kvs: vec![KeyValue {
                    key: Key::from_encoded("aGVsbG8"),
                    value: Value::from_encoded("dmFsdWU"),
                },]
            },
            deserialized
        );
    }

    #[test]
    fn key_value_range_serialize() {
        let serialized = serde_json::to_string(&KeyValueRange {
            next_key: Some(Key::from_encoded("bmV4dCBrZX")),
            kvs: vec![KeyValue {
                key: Key::from_encoded("aGVsbG8"),
                value: Value::from_encoded("dmFsdWU"),
            }],
        })
        .unwrap();

        assert_eq!(
            "{\"next_key\":\"bmV4dCBrZX\",\"kvs\":[{\"key\":\"aGVsbG8\",\"value\":\"dmFsdWU\"}]}",
            serialized
        );
    }
}
