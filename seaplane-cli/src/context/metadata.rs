use std::{
    fs::File,
    io::{self, Read},
};

use base64::{
    alphabet::URL_SAFE,
    engine::{general_purpose::NO_PAD, Engine, GeneralPurpose},
};
use seaplane::api::{metadata::v1::Key, shared::v1::Directory};

use crate::{
    cli::cmds::metadata::{SeaplaneMetadataCommonArgMatches, SeaplaneMetadataSetArgMatches},
    error::{CliError, Context, Result},
    ops::metadata::{KeyValue, KeyValues},
    printer::Color,
};

/// Represents the "Source of Truth" i.e. it combines all the CLI options, ENV vars, and config
/// values into a single structure that can be used later to build models for the API or local
/// structs for serializing
// TODO: we may not want to derive this we implement circular references
#[derive(Debug, Default, Clone)]
pub struct MetadataCtx {
    pub kvs: KeyValues,
    pub directory: Option<Directory>,
    /// Is the key or value already URL safe base64 encoded
    pub base64: bool,
    /// Print with decoding
    pub decode: bool,
    /// Print with safe decoding
    pub decode_safe: bool,
    /// Print with no decoding
    pub no_decode: bool,
    /// A base64 encoded key
    pub from: Option<Key>,
    /// Skip the KEY or VALUE header in --format=table
    pub no_header: bool,
    /// Don't print keys
    pub no_keys: bool,
    /// Don't print values
    pub no_values: bool,
    /// Max width of keys
    pub keys_width_limit: usize,
    /// Max width of values
    pub values_width_limit: usize,
}

impl MetadataCtx {
    /// Builds a MetadataCtx from ArgMatches
    pub fn from_md_common(matches: &SeaplaneMetadataCommonArgMatches) -> Result<MetadataCtx> {
        let matches = matches.0;
        let base64 = matches.get_flag("base64");
        let raw_keys: Vec<_> = matches.get_many::<String>("key").unwrap().collect();

        let mut kvs = KeyValues::default();
        for key in raw_keys {
            if base64 {
                // Check that what the user passed really is valid base64
                let engine = GeneralPurpose::new(&URL_SAFE, NO_PAD);
                let _unused = engine.decode(key)?;
                kvs.push(KeyValue::from_key(key));
            } else {
                kvs.push(KeyValue::from_key_unencoded(key));
            }
        }

        Ok(MetadataCtx {
            kvs,
            base64: true, // At this point all keys and values should be encoded as base64
            ..MetadataCtx::default()
        })
    }

    /// Builds a MetadataCtx from ArgMatches
    pub fn from_md_set(matches: &SeaplaneMetadataSetArgMatches) -> Result<MetadataCtx> {
        let matches = matches.0;
        let base64 = matches.get_flag("base64");
        let raw_key = matches.get_one::<String>("key").unwrap();
        let raw_value = matches.get_one::<String>("value").unwrap();
        let value = if let Some(val) = raw_value.strip_prefix('@') {
            if val == "-" {
                let mut buf: Vec<u8> = Vec::new();
                let stdin = io::stdin();
                let mut stdin_lock = stdin.lock();
                stdin_lock.read_to_end(&mut buf)?;
                buf
            } else {
                let mut f = File::open(val)
                    .map_err(CliError::from)
                    .context("\n\tpath: ")
                    .with_color_context(|| (Color::Yellow, val))?;

                // TODO: @perf we could pre-allocate the vec based on the file size
                let mut buf = Vec::new();

                f.read_to_end(&mut buf)?;
                buf
            }
        } else {
            raw_value.as_bytes().to_vec()
        };

        let engine = GeneralPurpose::new(&URL_SAFE, NO_PAD);
        let kv = if base64 {
            // make sure it's valid base64
            let _unused = engine.decode(raw_key)?;
            let _unused = engine.decode(&value)?;
            // The user used `--base64` and it is valid base64 so there is no reason the from_utf8
            // should fail
            KeyValue::new(raw_key, &String::from_utf8(value)?)
        } else {
            KeyValue::new(engine.encode(raw_key), engine.encode(value))
        };

        let mut kvs = KeyValues::default();
        kvs.push(kv);

        Ok(MetadataCtx { kvs, base64: true, ..MetadataCtx::default() })
    }
}
