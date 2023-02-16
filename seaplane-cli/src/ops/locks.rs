use std::io::Write;

use base64::{
    alphabet::URL_SAFE,
    engine::{general_purpose::NO_PAD, Engine, GeneralPurpose},
};
use seaplane::api::locks::v1::{LockInfo, LockInfoInner, LockName as LockNameModel};
use serde::Serialize;
use tabwriter::TabWriter;

use crate::{
    context::Ctx,
    error::{CliError, Result},
    ops::EncodedString,
    printer::{printer, Output},
};

/// We use our own LockName instead of the models because we need to *not* enforce base64 encoding,
/// and implement a bunch of additional methods and traits that wouldn't make sense for the models
///
/// We also need to keep track if the values are encoded or not
#[derive(Debug, Default, Clone, Serialize)]
pub struct LockName {
    pub name: EncodedString,
}

impl LockName {
    /// Creates a new LockName from an encoded name. You must pinky promise the name
    /// is URL safe base64 encoded or Bad Things may happen.
    pub fn new<S: Into<String>>(name: S) -> Self { Self { name: EncodedString::new(name.into()) } }

    /// Creates a new LockName from an un-encoded byte slice ref, encoding it along the way
    pub fn from_name_unencoded<S: AsRef<[u8]>>(name: S) -> Self {
        let engine = GeneralPurpose::new(&URL_SAFE, NO_PAD);
        let name = engine.encode(name.as_ref());
        Self { name: EncodedString::new(name) }
    }

    /// Creates a new LockName from self's data.
    pub fn to_model(&self) -> LockNameModel { LockNameModel::from_encoded(self.name.to_string()) }
}

#[derive(Debug, Serialize)]
pub struct HeldLock {
    pub lock_id: String,
    pub sequencer: u32,
}

impl Output for HeldLock {
    fn print_json(&self, _ctx: &Ctx) -> Result<()> {
        cli_println!("{}", serde_json::to_string(self)?);
        Ok(())
    }

    fn print_table(&self, ctx: &Ctx) -> Result<()> {
        let show_headers = !ctx.locks_ctx.get_or_init().no_header;
        let mut ptr = printer();

        let id_prefix = if show_headers { "LOCK-ID: " } else { "" };
        let seq_prefix = if show_headers { "SEQUENCER: " } else { "" };
        writeln!(ptr, "{id_prefix}{}", self.lock_id)?;
        writeln!(ptr, "{seq_prefix}{}", self.sequencer)?;

        ptr.flush()?;

        Ok(())
    }
}

#[derive(Debug, Serialize)]
pub struct ListedLockInfoInner {
    pub ttl: u32,
    #[serde(rename = "client-id")]
    pub client_id: String,
    pub ip: String,
}

impl From<LockInfoInner> for ListedLockInfoInner {
    fn from(other: LockInfoInner) -> Self {
        Self { ttl: other.ttl, client_id: other.client_id, ip: other.ip }
    }
}

#[derive(Debug, Serialize)]
pub struct ListedLock {
    name: EncodedString,
    id: String,
    info: ListedLockInfoInner,
}

impl From<LockInfo> for ListedLock {
    fn from(other: LockInfo) -> Self {
        let info = other.info.into();
        Self {
            name: EncodedString::new(other.name.encoded().to_owned()),
            id: other.id.encoded().to_owned(),
            info,
        }
    }
}

pub fn print_lock_table<I>(headers: bool, chunk: I, ctx: &Ctx) -> Result<()>
where
    I: IntoIterator<Item = ListedLock>,
{
    let buffer = Vec::new();
    let mut tw = TabWriter::new(buffer);
    if headers {
        writeln!(tw, "LOCK-NAME\tLOCK-ID\tCLIENT-ID\tCLIENT-IP\tTTL")?;
    }

    let locksctx = ctx.locks_ctx.get_or_init();
    for l in chunk {
        if locksctx.decode {
            // decoded names with tabs in them are going to act funny
            // (workaround is to not decode them)
            tw.write_all(&l.name.decoded()?)?;
        } else {
            write!(tw, "{}", l.name)?;
        };

        writeln!(tw, "\t{}\t{}\t{}\t{}", l.id, l.info.client_id, l.info.ip, l.info.ttl)?;
    }
    tw.flush()?;

    let mut ptr = printer();
    let page = tw
        .into_inner()
        .map_err(|_| CliError::bail("IO flush error writing locks"))?;
    ptr.write_all(&page)?;
    ptr.flush()?;

    Ok(())
}
