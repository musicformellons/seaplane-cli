use std::io::Write;

use base64::{
    alphabet::URL_SAFE,
    engine::{general_purpose::NO_PAD, Engine, GeneralPurpose},
};
use seaplane::api::metadata::v1::KeyValue as KeyValueModel;
use serde::Serialize;
use tabwriter::TabWriter;
use unicode_segmentation::UnicodeSegmentation;

use crate::{
    context::Ctx,
    error::{CliError, Result},
    ops::EncodedString,
    printer::{printer, Output},
};

/// We use our own KeyValue instead of the models because we need to *not* enforce base64 encoding,
/// and implement a bunch of additional methods and traits that wouldn't make sense for the models
///
/// We also need to keep track if the values are encoded or not
#[derive(Debug, Default, Clone, Serialize)]
pub struct KeyValue {
    pub key: EncodedString,
    pub value: EncodedString,
}

impl KeyValue {
    /// Creates a new KeyValue from an encoded key and value. You must pinky promise the key and
    /// value are URL safe base64 encoded or Bad Things may happen.
    pub fn new<S: Into<String>>(key: S, value: S) -> Self {
        Self { key: EncodedString::new(key.into()), value: EncodedString::new(value.into()) }
    }

    /// Creates a new KeyValue from an un-encoded key and value, encoding them along the way
    pub fn new_unencoded<S: AsRef<str>>(key: S, value: S) -> Self {
        let engine = GeneralPurpose::new(&URL_SAFE, NO_PAD);
        Self::new(engine.encode(key.as_ref()), engine.encode(value.as_ref()))
    }

    /// Creates a new KeyValue from an un-encoded string ref, encoding it along the way
    pub fn from_key_unencoded<S: AsRef<str>>(key: S) -> Self {
        let engine = GeneralPurpose::new(&URL_SAFE, NO_PAD);
        Self::from_key(engine.encode(key.as_ref()))
    }

    /// Creates a new KeyValue from an already encoded string ref. You must pinky promise the key
    /// is URL safe base64 encoded or Bad Things may happen.
    pub fn from_key<S: Into<String>>(key: S) -> Self {
        Self { key: EncodedString::new(key.into()), ..Self::default() }
    }

    /// Sets the value to some base64 encoded value
    pub fn set_value<S: Into<String>>(&mut self, value: S) {
        self.value = EncodedString::new(value.into())
    }
}

#[derive(Debug, Default, Clone, Serialize)]
#[serde(transparent)]
pub struct KeyValues {
    inner: Vec<KeyValue>,
}

impl KeyValues {
    pub fn from_model(model: Vec<KeyValueModel>) -> Self {
        Self {
            inner: model
                .iter()
                .map(|kv| KeyValue::new(kv.key.as_ref(), kv.value.as_ref()))
                .collect(),
        }
    }

    /// Inserts an already base64 encoded key and value
    pub fn insert<S: Into<String>>(&mut self, key: S, value: S) {
        self.inner.push(KeyValue::new(key, value));
    }

    pub fn iter(&self) -> impl Iterator<Item = &KeyValue> { self.inner.iter() }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut KeyValue> { self.inner.iter_mut() }

    pub fn push(&mut self, kv: KeyValue) { self.inner.push(kv) }

    pub fn keys(&self) -> impl Iterator<Item = EncodedString> + '_ {
        self.inner.iter().map(|kv| kv.key.clone())
    }
}

impl Output for KeyValues {
    fn print_json(&self, _ctx: &Ctx) -> Result<()> {
        cli_println!("{}", serde_json::to_string(self)?);
        Ok(())
    }

    fn print_table(&self, ctx: &Ctx) -> Result<()> {
        let mdctx = ctx.md_ctx.get_or_init();
        let headers = !mdctx.no_header;
        let decode = mdctx.decode;
        let decode_safe = mdctx.decode_safe;
        let only_keys = mdctx.no_values;
        let only_values = mdctx.no_keys;
        let keys_width_limit = mdctx.keys_width_limit;
        let values_width_limit = mdctx.values_width_limit;

        let mut tw = TabWriter::new(Vec::new());

        if headers {
            match [only_keys, only_values] {
                [true, false] => writeln!(tw, "KEY")?,
                [false, true] => writeln!(tw, "VALUE")?,
                [..] => writeln!(tw, "KEY\tVALUE")?,
            }
        }

        // Truncate a binary vector to a specific length in bytes.
        // Appends ellipsis `…` if the vector was truncated
        fn truncate_vec(vec: Vec<u8>, max: usize) -> Vec<u8> {
            if max == 0 || vec.len() <= max {
                vec
            } else {
                let mut vec = vec;
                vec.truncate(max);
                vec.extend_from_slice("…".as_bytes());
                vec
            }
        }

        // Truncate a unicode string to a specific number of grapheme clusters.
        // Appends ellipsis `…` if the string was truncated
        fn truncate_string(str: String, max: usize) -> String {
            if max == 0 || str.graphemes(true).count() <= max {
                str
            } else {
                let mut s: String = str.graphemes(true).take(max).collect();
                s.push('…');
                s
            }
        }

        for kv in self.iter() {
            if !only_values {
                if decode {
                    tw.write_all(&truncate_vec(kv.key.decoded()?, keys_width_limit))?;
                } else if decode_safe {
                    write!(tw, "{}", truncate_string(kv.key.decoded_safe()?, keys_width_limit))?;
                } else {
                    write!(tw, "{}", truncate_string(kv.key.to_string(), keys_width_limit))?;
                }

                if !only_keys {
                    tw.write_all(b"\t")?
                };
            };

            if !only_keys {
                if decode {
                    tw.write_all(&truncate_vec(kv.value.decoded()?, values_width_limit))?;
                } else if decode_safe {
                    write!(
                        tw,
                        "{}",
                        truncate_string(kv.value.decoded_safe()?, values_width_limit)
                    )?;
                } else {
                    write!(tw, "{}", truncate_string(kv.value.to_string(), values_width_limit))?;
                }
            };
            writeln!(tw)?;
        }
        tw.flush()?;

        let mut ptr = printer();
        let page = tw
            .into_inner()
            .map_err(|_| CliError::bail("IO flush error writing key-values"))?;
        ptr.write_all(&page)?;
        ptr.flush()?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    fn build_kvs() -> KeyValues {
        KeyValues {
            inner: vec![
                KeyValue {
                    key: EncodedString::new("a2V5MQ".into()),
                    value: EncodedString::new("dmFsdWUx".into()),
                },
                KeyValue {
                    key: EncodedString::new("a2V5Mg".into()),
                    value: EncodedString::new("dmFsdWUy".into()),
                },
                KeyValue {
                    key: EncodedString::new("a2V5Mw".into()),
                    value: EncodedString::new("dmFsdWUz".into()),
                },
            ],
        }
    }

    #[test]
    fn serialize_keyvalues_base64() {
        let kvs = build_kvs();

        assert_eq!(
            serde_json::to_string(&kvs).unwrap(),
            json!([{"key": "a2V5MQ", "value": "dmFsdWUx"}, {"key": "a2V5Mg", "value": "dmFsdWUy"}, {"key": "a2V5Mw", "value": "dmFsdWUz"}]).to_string()
        );
    }
}
