use std::{
    borrow::Cow,
    io,
    result::Result as StdResult,
    sync::{Mutex, MutexGuard, PoisonError},
    time::Duration,
};

use indicatif::{ProgressBar, ProgressStyle};
use once_cell::sync::OnceCell;
use serde::{
    de::{self, Deserializer},
    Deserialize, Serialize,
};
use strum::{Display, EnumString};

use crate::{error::Result, Ctx};

#[cfg_attr(feature = "api_tests", allow(dead_code))]
static GLOBAL_PRINTER: OnceCell<Mutex<self::_printer::Printer>> = OnceCell::new();
#[cfg_attr(feature = "api_tests", allow(dead_code))]
static GLOBAL_EPRINTER: OnceCell<Mutex<self::_printer::Printer>> = OnceCell::new();

pub use self::_printer::{eprinter, printer, Printer};

/// We wrap the progress bar to be able to hide output when we need to
#[allow(missing_debug_implementations)]
pub struct Pb(Option<ProgressBar>);

impl Pb {
    pub fn new(ctx: &Ctx) -> Self {
        if !ctx.disable_pb && crate::log::log_level() <= &crate::log::LogLevel::Info {
            let pb = ProgressBar::new_spinner();
            pb.enable_steady_tick(Duration::from_millis(120));
            pb.set_style(
                ProgressStyle::default_bar()
                    .template("{spinner:.green} {msg}")
                    .expect("Invalid progress bar template"),
            );
            Pb(Some(pb))
        } else {
            Pb(None)
        }
    }

    pub fn set_message(&self, msg: impl Into<Cow<'static, str>>) {
        if let Some(pb) = &self.0 {
            pb.set_message(msg);
        }
    }

    pub fn finish_and_clear(&self) {
        if let Some(pb) = &self.0 {
            pb.finish_and_clear()
        }
    }
}

impl Drop for Pb {
    fn drop(&mut self) { self.finish_and_clear() }
}

#[derive(EnumString, Display, Deserialize, Copy, Clone, Debug, PartialEq, Eq, clap::ValueEnum)]
#[strum(ascii_case_insensitive, serialize_all = "lowercase")]
#[derive(Default)]
pub enum OutputFormat {
    #[default]
    Table,
    Json,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, clap::ValueEnum, Display, EnumString)]
#[strum(ascii_case_insensitive, serialize_all = "lowercase")]
#[derive(Default)]
pub enum ColorChoice {
    Always,
    Ansi,
    #[default]
    Auto,
    Never,
}

impl<'de> Deserialize<'de> for ColorChoice {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> StdResult<Self, D::Error> {
        use std::str::FromStr;
        let s = <String>::deserialize(deserializer)?;
        ColorChoice::from_str(&s).map_err(de::Error::custom)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
#[allow(dead_code)]
pub enum Color {
    Black,
    Blue,
    Green,
    Red,
    Cyan,
    Magenta,
    Yellow,
    White,
}

pub trait Output {
    fn print_json(&self, _ctx: &Ctx) -> Result<()> {
        cli_eprint!("--format=");
        cli_eprint!(@Yellow, "json");
        cli_eprintln!(" is not supported by this object");

        Ok(())
    }

    fn print_table(&self, _ctx: &Ctx) -> Result<()> {
        cli_eprint!("--format=");
        cli_eprint!(@Yellow, "table");
        cli_eprintln!(" is not supported by this object");

        Ok(())
    }
}

#[cfg(all(feature = "color", not(feature = "api_tests")))]
mod _printer {
    use atty::Stream;
    use termcolor::{
        Color as TermColorColor, ColorChoice as TermColorChoice, ColorSpec, StandardStream,
        WriteColor,
    };

    use super::*;

    fn detect_tty(stream: Stream) -> TermColorChoice {
        if atty::is(stream) {
            TermColorChoice::Auto
        } else {
            TermColorChoice::Never
        }
    }

    #[allow(missing_debug_implementations)]
    pub struct Printer(StandardStream);

    pub fn printer() -> MutexGuard<'static, Printer> {
        GLOBAL_PRINTER
            .get_or_init(|| Mutex::new(Printer(StandardStream::stdout(TermColorChoice::Auto))))
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
    }

    pub fn eprinter() -> MutexGuard<'static, Printer> {
        GLOBAL_EPRINTER
            .get_or_init(|| Mutex::new(Printer(StandardStream::stderr(TermColorChoice::Auto))))
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
    }

    impl Printer {
        pub fn init(color: ColorChoice) {
            use TermColorChoice::*;
            let (choice, echoice) = match color {
                ColorChoice::Always => (Always, Always),
                ColorChoice::Auto => (detect_tty(Stream::Stdout), detect_tty(Stream::Stderr)),
                ColorChoice::Ansi => (AlwaysAnsi, AlwaysAnsi),
                ColorChoice::Never => (Never, Never),
            };

            printer().set_stream(StandardStream::stdout(choice));
            eprinter().set_stream(StandardStream::stderr(echoice));
        }

        fn set_stream(&mut self, stream: StandardStream) { self.0 = stream; }

        pub fn set_color(&mut self, color: Color) {
            let _ = self
                .0
                .set_color(ColorSpec::new().set_fg(Some(color.into_termcolor())));
        }

        pub fn reset(&mut self) { let _ = self.0.reset(); }
    }

    impl io::Write for Printer {
        fn write(&mut self, buf: &[u8]) -> io::Result<usize> { self.0.write(buf) }

        fn flush(&mut self) -> io::Result<()> { self.0.flush() }
    }

    impl Color {
        fn into_termcolor(self) -> TermColorColor {
            match self {
                Color::Black => TermColorColor::Black,
                Color::Blue => TermColorColor::Blue,
                Color::Green => TermColorColor::Green,
                Color::Red => TermColorColor::Red,
                Color::Cyan => TermColorColor::Cyan,
                Color::Magenta => TermColorColor::Magenta,
                Color::Yellow => TermColorColor::Yellow,
                Color::White => TermColorColor::White,
            }
        }
    }
}

#[cfg(all(not(feature = "color"), not(feature = "api_tests")))]
mod _printer {
    use super::*;

    enum StandardStream {
        Stdout(io::Stdout),
        Stderr(io::Stderr),
    }

    #[allow(missing_debug_implementations)]
    pub struct Printer(StandardStream);

    pub fn printer() -> MutexGuard<'static, Printer> {
        GLOBAL_PRINTER
            .get_or_init(|| Mutex::new(Printer(StandardStream::Stdout(io::stdout()))))
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
    }

    pub fn eprinter() -> MutexGuard<'static, Printer> {
        GLOBAL_EPRINTER
            .get_or_init(|| Mutex::new(Printer(StandardStream::Stderr(io::stderr()))))
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
    }

    impl Printer {
        pub fn init(_color: ColorChoice) {
            let _a = printer();
            let _a = eprinter();
        }

        pub fn set_color(&mut self, _color: Color) {}

        pub fn reset(&mut self) {}
    }

    impl io::Write for Printer {
        fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
            match self.0 {
                StandardStream::Stdout(ref mut s) => s.write(buf),
                StandardStream::Stderr(ref mut s) => s.write(buf),
            }
        }

        fn flush(&mut self) -> io::Result<()> {
            match self.0 {
                StandardStream::Stdout(ref mut s) => s.flush(),
                StandardStream::Stderr(ref mut s) => s.flush(),
            }
        }
    }
}

#[cfg(feature = "api_tests")]
mod _printer {
    use std::borrow::Cow;

    use super::*;

    static GLOBAL_TEST_PRINTER: OnceCell<Mutex<self::_printer::Printer>> = OnceCell::new();
    static GLOBAL_TEST_EPRINTER: OnceCell<Mutex<self::_printer::Printer>> = OnceCell::new();

    #[allow(missing_debug_implementations)]
    pub struct Printer(pub Vec<u8>);

    pub fn printer() -> MutexGuard<'static, Printer> {
        GLOBAL_TEST_PRINTER
            .get_or_init(|| Mutex::new(Printer(Vec::new())))
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
    }

    pub fn eprinter() -> MutexGuard<'static, Printer> {
        GLOBAL_TEST_EPRINTER
            .get_or_init(|| Mutex::new(Printer(Vec::new())))
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
    }

    impl Printer {
        pub fn init(_color: ColorChoice) {
            let _a = printer();
            let _a = eprinter();
        }

        pub fn set_color(&mut self, _color: Color) {}

        pub fn reset(&mut self) {}

        pub fn clear(&mut self) { self.0.clear() }

        pub fn as_string(&self) -> Cow<'_, str> { String::from_utf8_lossy(&self.0) }
    }

    impl io::Write for Printer {
        fn write(&mut self, buf: &[u8]) -> io::Result<usize> { self.0.write(buf) }

        fn flush(&mut self) -> io::Result<()> { Ok(()) }
    }
}
