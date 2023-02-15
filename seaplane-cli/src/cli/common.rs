use clap::{value_parser, Arg};

use crate::OutputFormat;

pub fn format() -> Arg {
    arg!(--format =["FORMAT"=>"table"])
        .value_parser(value_parser!(OutputFormat))
        .help("Change the output format")
}
