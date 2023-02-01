use std::{
    error::Error,
    io::{self, Write},
    path::PathBuf,
    result::Result as StdResult,
};

use seaplane::{
    api::ApiErrorKind, error::SeaplaneError, rexports::container_image_ref::ImageReferenceError,
};

use crate::{
    log::{log_level, LogLevel},
    printer::{eprinter, Color},
};

pub type Result<T> = StdResult<T, CliError>;

/// A trait for adding context to an error that will be printed along with the error. Contexts are
/// useful for adding things such as hints (i.e. try --help), or additional information such as the
/// path name on a PermissionDenied error, etc.
///
/// **NOTE:** all contexts print *without* a trailing newline. This allows a context to print to
/// the same line in different formats (colors, etc.). If a trailing newline is required, you
/// should add it manually.
pub trait Context {
    /// A simple context
    fn context<S: Into<String>>(self, msg: S) -> Self;

    /// A context that is evaluated lazily when called. This is useful if building the context is
    /// expensive or allocates
    fn with_context<F, S>(self, f: F) -> Self
    where
        F: FnOnce() -> S,
        S: Into<String>;

    /// A simple context that will color the output
    ///
    /// **NOTE:** The color is reset at the end of this context even if there is no trailing
    /// newline. This allows you to chain multiple contexts on the same line where only part of the
    /// context is colored.
    fn color_context<S: Into<String>>(self, color: Color, msg: S) -> Self;

    /// A context that will color the output and that is evaluated lazily when called. This is
    /// useful if building the context is expensive or allocates
    ///
    /// **NOTE:** The color is reset at the end of this context even if there is no trailing
    /// newline. This allows you to chain multiple contexts on the same line where only part of the
    /// context is colored.
    fn with_color_context<F, S>(self, f: F) -> Self
    where
        F: FnOnce() -> (Color, S),
        S: Into<String>;
}

impl<T> Context for StdResult<T, CliError> {
    fn context<S: Into<String>>(self, msg: S) -> Self {
        match self {
            Ok(t) => Ok(t),
            Err(cli_err) => Err(cli_err.context(msg)),
        }
    }
    fn color_context<S: Into<String>>(self, color: Color, msg: S) -> Self {
        match self {
            Ok(t) => Ok(t),
            Err(cli_err) => Err(cli_err.color_context(color, msg)),
        }
    }
    fn with_context<F, S>(self, f: F) -> Self
    where
        F: FnOnce() -> S,
        S: Into<String>,
    {
        match self {
            Ok(t) => Ok(t),
            Err(cli_err) => Err(cli_err.context(f())),
        }
    }

    fn with_color_context<F, S>(self, f: F) -> Self
    where
        F: FnOnce() -> (Color, S),
        S: Into<String>,
    {
        match self {
            Ok(t) => Ok(t),
            Err(cli_err) => {
                let (color, msg) = f();
                Err(cli_err.color_context(color, msg))
            }
        }
    }
}

#[derive(Debug)]
pub struct ColorString {
    msg: String,
    color: Option<Color>,
}

#[derive(Debug)]
pub struct CliError {
    kind: CliErrorKind,
    context: Vec<ColorString>,
    status: Option<i32>, // TODO: default to 1
}

impl CliError {
    pub fn bail(msg: &'static str) -> Self {
        Self { kind: CliErrorKind::UnknownWithContext(msg), ..Default::default() }
    }
}

impl Context for CliError {
    fn color_context<S: Into<String>>(mut self, color: Color, msg: S) -> Self {
        self.context
            .push(ColorString { msg: msg.into(), color: Some(color) });
        self
    }

    fn context<S: Into<String>>(mut self, msg: S) -> Self {
        self.context
            .push(ColorString { msg: msg.into(), color: None });
        self
    }

    fn with_context<F, S>(mut self, f: F) -> Self
    where
        F: FnOnce() -> S,
        S: Into<String>,
    {
        self.context
            .push(ColorString { msg: f().into(), color: None });
        self
    }

    fn with_color_context<F, S>(mut self, f: F) -> Self
    where
        F: FnOnce() -> (Color, S),
        S: Into<String>,
    {
        let (color, msg) = f();
        self.context
            .push(ColorString { msg: msg.into(), color: Some(color) });
        self
    }
}

impl Default for CliError {
    fn default() -> Self { Self { kind: CliErrorKind::Unknown, context: Vec::new(), status: None } }
}

// We have to impl Display so we can use the ? operator...but we don't actually want to use it's
// pipeline to do any kind of displaying because it doesn't support any sort of coloring. So we
// handle it manually.
impl std::fmt::Display for CliError {
    fn fmt(&self, _f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        panic!("std::fmt::Display is not actually implemented for CliError by design, use CliError::print instead")
    }
}

// Just so we can us the ? operator
impl Error for CliError {}

macro_rules! impl_err {
    (@boxed, $errty:ty, $variant:ident) => {
        impl From<$errty> for CliError {
            fn from(e: $errty) -> Self {
                CliError {
                    kind: CliErrorKind::$variant(std::boxed::Box::new(e)),
                    ..Default::default()
                }
            }
        }
    };
    ($errty:ty, $variant:ident) => {
        impl From<$errty> for CliError {
            fn from(e: $errty) -> Self {
                CliError { kind: CliErrorKind::$variant(e), ..Default::default() }
            }
        }
    };
}

// These are placeholders until we get around to writing distinct errors for the cases we care
// about
impl_err!(base64::DecodeError, Base64Decode);
impl_err!(serde_json::Error, SerdeJson);
impl_err!(@boxed, toml::de::Error, TomlDe);
impl_err!(@boxed, toml::ser::Error, TomlSer);
impl_err!(seaplane::error::SeaplaneError, Seaplane);
impl_err!(seaplane::rexports::container_image_ref::ImageReferenceError, ImageReference);
impl_err!(std::string::FromUtf8Error, InvalidUtf8);
impl_err!(hex::FromHexError, HexDecode);
impl_err!(std::num::ParseIntError, ParseInt);
impl_err!(strum::ParseError, StrumParse);
impl_err!(clap::Error, Clap);

impl From<io::Error> for CliError {
    fn from(e: io::Error) -> Self {
        match e.kind() {
            io::ErrorKind::NotFound => {
                CliError { kind: CliErrorKind::MissingPath, ..Default::default() }
            }
            io::ErrorKind::PermissionDenied => {
                CliError { kind: CliErrorKind::PermissionDenied, ..Default::default() }
            }
            _ => CliError { kind: CliErrorKind::Io(e, None), ..Default::default() },
        }
    }
}

impl From<tempfile::PersistError> for CliError {
    fn from(e: tempfile::PersistError) -> Self {
        Self {
            kind: CliErrorKind::Io(e.error, Some(e.file.path().to_path_buf())),
            ..Default::default()
        }
    }
}

impl From<tempfile::PathPersistError> for CliError {
    fn from(e: tempfile::PathPersistError) -> Self {
        Self { kind: CliErrorKind::Io(e.error, Some(e.path.to_path_buf())), ..Default::default() }
    }
}

impl From<CliErrorKind> for CliError {
    fn from(kind: CliErrorKind) -> Self { CliError { kind, ..Default::default() } }
}

#[derive(Debug)]
pub enum CliErrorKind {
    AmbiguousItem(String),
    Base64Decode(base64::DecodeError),
    Clap(clap::Error),
    CliArgNotUsed(&'static str),
    ConflictingArguments(String, String),
    DuplicateName(String),
    EndpointInvalidFlight(String),
    ExistingValue(&'static str),
    FlightsInUse(Vec<String>),
    HexDecode(hex::FromHexError),
    ImageReference(ImageReferenceError),
    InlineFlightHasSpace,
    InlineFlightInvalidName(String),
    InlineFlightMissingImage,
    InlineFlightMissingValue(String),
    InlineFlightUnknownItem(String),
    InvalidCliValue(Option<&'static str>, String),
    InvalidUtf8(std::string::FromUtf8Error),
    Io(io::Error, Option<PathBuf>),
    MissingApiKey,
    MissingPath,
    MultipleAtStdin,
    NoMatchingItem(String),
    OneOff(String),
    ParseInt(std::num::ParseIntError),
    PermissionDenied,
    Seaplane(SeaplaneError),
    SerdeJson(serde_json::Error),
    StrumParse(strum::ParseError),
    TomlDe(Box<toml::de::Error>),
    TomlSer(Box<toml::ser::Error>),
    Unknown,
    UnknownWithContext(&'static str),
}

impl CliErrorKind {
    fn print(&self) {
        use CliErrorKind::*;

        match self {
            OneOff(msg) => {
                cli_eprintln!("{msg}");
            }
            FlightsInUse(flights) => {
                cli_eprintln!("the following Flight Plans are referenced by a Formation Plan and cannot be deleted");
                for f in flights {
                    cli_eprintln!(@Yellow, "\t{f}");
                }
                cli_eprint!("\n(hint: override this check and force delete with '");
                cli_eprint!(@Yellow, "--force");
                cli_eprintln!("' which will also remove the Flight Plan from the Formation Plan)");
            }
            EndpointInvalidFlight(flight) => {
                cli_eprint!("Flight Plan '");
                cli_eprint!(@Red, "{flight}");
                cli_eprintln!(
                    "' is referenced in an endpoint but does not exist in the local Plans"
                );
            }
            ConflictingArguments(a, b) => {
                cli_eprint!("cannot use '");
                cli_eprint!(@Yellow, "{a}");
                cli_eprint!("' with '");
                cli_eprint!(@Yellow, "{b}");
                cli_eprintln!(
                    "'\n(hint: one or both arguments may have been implied from other flags)"
                );
            }
            Base64Decode(e) => {
                cli_eprintln!("base64 decode: {e}");
            }
            DuplicateName(name) => {
                cli_eprint!("an item with the name '");
                cli_eprint!(@Yellow, "{name}");
                cli_eprintln!("' already exists");
            }
            NoMatchingItem(item) => {
                cli_eprint!("the NAME or ID '");
                cli_eprint!(@Green, "{item}");
                cli_eprintln!("' didn't match anything");
            }
            AmbiguousItem(item) => {
                cli_eprint!("the NAME or ID '");
                cli_eprint!(@Yellow, "{item}");
                cli_eprintln!("' is ambiguous and matches more than one item");
            }
            MissingPath => {
                cli_eprintln!("missing file or directory");
            }
            PermissionDenied => {
                cli_eprintln!("permission denied when accessing file or directory");
            }
            HexDecode(e) => {
                cli_eprintln!("hex decode: {e}")
            }
            ImageReference(e) => {
                cli_eprintln!("seaplane: {e}")
            }
            InvalidUtf8(e) => {
                cli_eprintln!("invalid UTF-8: {e}")
            }
            StrumParse(e) => {
                cli_eprintln!("string parse error: {e}")
            }
            Io(e, Some(path)) => {
                cli_eprintln!("io: {e}");
                cli_eprint!("\tpath: ");
                cli_eprintln!(@Yellow, "{path:?}");
            }
            Io(e, None) => {
                cli_eprintln!("io: {e}");
            }
            SerdeJson(e) => {
                cli_eprintln!("json: {e}")
            }
            TomlDe(e) => {
                cli_eprintln!("toml: {e}")
            }
            TomlSer(e) => {
                cli_eprintln!("toml: {e}")
            }
            ParseInt(e) => {
                cli_eprintln!("parse integer: {e}")
            }
            UnknownWithContext(e) => {
                cli_eprintln!("unknown: {e}")
            }
            InvalidCliValue(a, v) => {
                cli_eprint!("the CLI value '");
                if let Some(val) = a {
                    cli_eprint!("--{val}=");
                    cli_eprint!(@Red, "{v}");
                } else {
                    cli_eprint!(@Red, "{v}");
                }
                cli_eprintln!("' isn't valid");
            }
            CliArgNotUsed(a) => {
                cli_eprint!("the CLI argument '");
                cli_eprint!("{a}");
                cli_eprintln!("' wasn't used but is required in this context");
            }
            Unknown => {
                cli_eprintln!("unknown")
            }
            MissingApiKey => {
                cli_eprintln!("no API key was found or provided")
            }
            MultipleAtStdin => {
                cli_eprint!("more than one '");
                cli_print!(@Yellow, "@-");
                cli_println!("' values were provided and only one is allowed");
            }
            Seaplane(e) => match e {
                SeaplaneError::ApiResponse(ae) => {
                    cli_eprintln!("{ae}");
                    if ae.kind == ApiErrorKind::BadRequest
                        && ae.message.contains("`force` flag was not set")
                    {
                        cli_eprint!("(hint: set the force parameter with '");
                        cli_eprint!(@Yellow, "--force");
                        cli_eprintln!("')");
                    }
                }
                _ => {
                    cli_eprintln!("Seaplane API: {e}")
                }
            },
            ExistingValue(value) => {
                cli_eprintln!("{value} already exists");
            }
            InlineFlightUnknownItem(item) => {
                cli_eprintln!("{item} is not a valid INLINE-FLIGHT-SPEC item (valid keys are: name, image, maximum, minimum, api-permission, architecture)");
            }
            InlineFlightInvalidName(name) => {
                cli_eprintln!("'{name}' is not a valid Flight Plan name");
            }
            InlineFlightHasSpace => {
                cli_eprintln!("INLINE-FLIGHT-SPEC contains a space ' ' which isn't allowed.");
            }
            InlineFlightMissingImage => {
                cli_eprintln!(
                    "INLINE-FLIGHT-SPEC missing image=... key and value which is required"
                );
            }
            InlineFlightMissingValue(key) => {
                cli_eprintln!("INLINE-FLIGHT-SPEC missing a value for the key {key}");
            }
            Clap(e) => {
                cli_eprintln!("{e}")
            }
        }
    }

    pub fn into_err(self) -> CliError { CliError { kind: self, ..Default::default() } }

    #[cfg(test)]
    pub fn is_parse_int(&self) -> bool { matches!(self, Self::ParseInt(_)) }
    #[cfg(test)]
    pub fn is_strum_parse(&self) -> bool { matches!(self, Self::StrumParse(_)) }
}

// Impl PartialEq manually so we can just match on kind, and not the associated data
impl PartialEq<Self> for CliErrorKind {
    fn eq(&self, rhs: &Self) -> bool {
        use CliErrorKind::*;

        match self {
            OneOff(_) => matches!(rhs, OneOff(_)),
            EndpointInvalidFlight(_) => matches!(rhs, EndpointInvalidFlight(_)),
            AmbiguousItem(_) => matches!(rhs, AmbiguousItem(_)),
            Io(_, _) => matches!(rhs, Io(_, _)),
            DuplicateName(_) => matches!(rhs, DuplicateName(_)),
            MissingApiKey => matches!(rhs, MissingApiKey),
            MissingPath => matches!(rhs, MissingPath),
            NoMatchingItem(_) => matches!(rhs, NoMatchingItem(_)),
            PermissionDenied => matches!(rhs, PermissionDenied),
            MultipleAtStdin => matches!(rhs, MultipleAtStdin),
            Seaplane(_) => matches!(rhs, Seaplane(_)),
            SerdeJson(_) => matches!(rhs, SerdeJson(_)),
            TomlSer(_) => matches!(rhs, TomlSer(_)),
            TomlDe(_) => matches!(rhs, TomlDe(_)),
            Unknown => matches!(rhs, Unknown),
            UnknownWithContext(_) => matches!(rhs, UnknownWithContext(_)),
            ExistingValue(_) => matches!(rhs, ExistingValue(_)),
            ImageReference(_) => matches!(rhs, ImageReference(_)),
            CliArgNotUsed(_) => matches!(rhs, CliArgNotUsed(_)),
            InvalidCliValue(_, _) => matches!(rhs, InvalidCliValue(_, _)),
            StrumParse(_) => matches!(rhs, StrumParse(_)),
            Base64Decode(_) => matches!(rhs, Base64Decode(_)),
            InvalidUtf8(_) => matches!(rhs, InvalidUtf8(_)),
            HexDecode(_) => matches!(rhs, HexDecode(_)),
            ConflictingArguments(_, _) => matches!(rhs, ConflictingArguments(_, _)),
            InlineFlightUnknownItem(_) => matches!(rhs, InlineFlightUnknownItem(_)),
            InlineFlightInvalidName(_) => matches!(rhs, InlineFlightInvalidName(_)),
            InlineFlightHasSpace => matches!(rhs, InlineFlightHasSpace),
            InlineFlightMissingImage => matches!(rhs, InlineFlightMissingImage),
            InlineFlightMissingValue(_) => matches!(rhs, InlineFlightMissingValue(_)),
            ParseInt(_) => matches!(rhs, ParseInt(_)),
            FlightsInUse(_) => matches!(rhs, FlightsInUse(_)),
            Clap(_) => matches!(rhs, Clap(_)),
        }
    }
}

impl CliError {
    /// Essentially destructure the cli_*! macros which actually also reduces the branches
    pub fn print(&self) {
        if log_level() <= &LogLevel::Error {
            // Scope for acquiring Mutex on global printer
            {
                let mut ptr = eprinter();
                ptr.set_color(Color::Red);
                let _ = write!(ptr, "error: ");
                ptr.reset();
            }

            // This function will try to reacquire the mutex
            self.kind.print();

            // Reacquire mutex lock
            let mut ptr = eprinter();
            for ColorString { color, msg } in &self.context {
                if let Some(c) = color {
                    ptr.set_color(*c);
                }
                let _ = write!(ptr, "{msg}");
                ptr.reset();
            }
        }
    }

    pub fn exit(&self) -> ! {
        self.print();
        // TODO: solidify what should happen if an error with self.fatal = false is called here...
        std::process::exit(self.status.unwrap_or(1))
    }

    pub fn kind(&self) -> &CliErrorKind { &self.kind }
}
