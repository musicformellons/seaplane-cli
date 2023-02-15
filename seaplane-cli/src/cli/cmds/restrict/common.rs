use clap::{builder::PossibleValue, value_parser, Arg, ArgMatches};
use seaplane::api::shared::v1::{Provider as ProviderModel, Region as RegionModel};

use crate::cli::common;

const LONG_DECODE: &str = "Decode the directories before printing them

Binary values will be written directly to standard output (which may do strange
things to your terminal)";

// TODO: Factor region and provider stuff as usual
static LONG_REGION: &str = "A region where the data placement is allowed (See REGION SPEC below)

Multiple items can be passed as a comma separated list, or by using the argument
multiple times.";

static LONG_PROVIDER: &str = "A provider where the data placement is allowed

Multiple items can be passed as a comma separated list, or by using the argument
multiple times.";

static LONG_EXCLUDE_PROVIDER: &str = "A provider where the data placement is *NOT* allowed

This will override any values given to --provider

Multiple items can be passed as a comma separated list, or by using the argument
multiple times.";

static LONG_EXCLUDE_REGION: &str =
    "A region  where the data placement is *NOT* allowed (See REGION SPEC below)

This will override any values given to --region

Multiple items can be passed as a comma separated list, or by using the argument
multiple times.";

// NOTE: we can't use `derive(clap::ValueEnum)` because it of how it derives the to_possible_value
// which appears to unconditionally use shish-ka-bob case which we don't want.
/// We provide a shim between the Seaplane Provider so we can do some additional UX work like 'all'
#[derive(Debug, Copy, Clone, PartialEq, Eq, strum::EnumString)]
#[strum(ascii_case_insensitive, serialize_all = "lowercase")]
pub enum Provider {
    Aws,
    Azure,
    DigitalOcean,
    Equinix,
    Gcp,
    All,
}

impl clap::ValueEnum for Provider {
    fn value_variants<'a>() -> &'a [Self] {
        use Provider::*;
        &[Aws, Azure, DigitalOcean, Equinix, Gcp, All]
    }

    fn to_possible_value<'a>(&self) -> Option<PossibleValue> {
        use Provider::*;
        match self {
            Aws => Some(PossibleValue::new("aws")),
            Azure => Some(PossibleValue::new("azure")),
            DigitalOcean => Some(PossibleValue::new("digitalocean")),
            Equinix => Some(PossibleValue::new("equinix")),
            Gcp => Some(PossibleValue::new("gcp")),
            All => Some(PossibleValue::new("all")),
        }
    }

    fn from_str(input: &str, _ignore_case: bool) -> Result<Self, String> {
        // Because we use strum(ascii_ignore_case) for our FromStr impl we unconditionally ignore
        // case and can ignore clap's hint
        input.parse().map_err(|e| format!("{e}"))
    }
}

impl Provider {
    pub fn into_model(&self) -> Option<ProviderModel> {
        if self == &Provider::All {
            None
        } else {
            Some(self.into())
        }
    }
}

#[allow(clippy::from_over_into)]
impl<'a> Into<ProviderModel> for &'a Provider {
    fn into(self) -> ProviderModel {
        use Provider::*;
        match self {
            Aws => ProviderModel::AWS,
            Azure => ProviderModel::Azure,
            DigitalOcean => ProviderModel::DigitalOcean,
            Equinix => ProviderModel::Equinix,
            Gcp => ProviderModel::GCP,
            All => {
                panic!("Provider::All cannot be converted into seaplane::api::shared::v1::Provider")
            }
        }
    }
}

// TODO: @needs-decision @pre-1.0 we need to come back and address how many aliases there are and
// their sort order for the CLI's "possible values" message. They're currently "grouped" in region
// aliases, but for one it's too many values, also it makes the sort look wild in the help message.
// The Compute API only uses the XA, XC, XE values, but those are the least user friendly.
/// We provide a shim between the Seaplane Region so we can do some additional UX work
#[allow(clippy::upper_case_acronyms)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, strum::EnumString)]
#[strum(ascii_case_insensitive, serialize_all = "lowercase")]
pub enum Region {
    XA,
    Asia,
    XC,
    PRC,
    PeoplesRepublicOfChina,
    XE,
    Europe,
    EU,
    XF,
    Africa,
    XN,
    NorthAmerica,
    NAmerica,
    XO,
    Oceania,
    XQ,
    Antarctica,
    XS,
    SAmerica,
    SouthAmerica,
    XU,
    UK,
    UnitedKingdom,
    All,
}

impl clap::ValueEnum for Region {
    fn value_variants<'a>() -> &'a [Self] {
        use Region::*;
        &[
            XA, // Asia,
            XC, // PRC, PeoplesRepublicOfChina,
            XE, // Europe, EU,
            XF, // Africa,
            XN, // NorthAmerica, NAmerica,
            XO, // Oceania,
            XQ, // Antarctica,
            XS, // SAmerica, SouthAmerica,
            XU, // UK, UnitedKingdom,
            All,
        ]
    }

    fn to_possible_value<'a>(&self) -> Option<PossibleValue> {
        use Region::*;
        match self {
            XA => Some(PossibleValue::new("xa").alias("asia")),
            Asia => None,
            XC => Some(PossibleValue::new("xc").aliases([
                "prc",
                "china",
                "peoples-republic-of-china",
                "peoples_republic_of_china",
                "peoplesrepublicofchina",
            ])),
            PRC => None,
            PeoplesRepublicOfChina => None,
            XE => Some(PossibleValue::new("xe").aliases(["europe", "eu"])),
            Europe => None,
            EU => None,
            XF => Some(PossibleValue::new("xf").alias("africa")),
            Africa => None,
            XN => Some(PossibleValue::new("xn").aliases([
                "namerica",
                "northamerica",
                "n-america",
                "north-america",
                "n_america",
                "north_america",
            ])),
            NorthAmerica => None,
            NAmerica => None,
            XO => Some(PossibleValue::new("xo").alias("oceania")),
            Oceania => None,
            XQ => Some(PossibleValue::new("xq").alias("antarctica")),
            Antarctica => None,
            XS => Some(PossibleValue::new("xs").aliases([
                "samerica",
                "southamerica",
                "s-america",
                "south-america",
                "s_america",
                "south_america",
            ])),
            SAmerica => None,
            SouthAmerica => None,
            XU => Some(PossibleValue::new("xu").aliases([
                "uk",
                "unitedkingdom",
                "united-kingdom",
                "united_kingdom",
            ])),
            UK => None,
            UnitedKingdom => None,
            All => Some(PossibleValue::new("all")),
        }
    }

    fn from_str(input: &str, _ignore_case: bool) -> Result<Self, String> {
        // Because we use strum(ascii_ignore_case) for our FromStr impl we unconditionally ignore
        // case and can ignore clap's hint
        input.parse().map_err(|e| format!("{e}"))
    }
}

impl Region {
    pub fn into_model(&self) -> Option<RegionModel> {
        if self == &Region::All {
            None
        } else {
            Some(self.into())
        }
    }
}

#[allow(clippy::from_over_into)]
impl<'a> Into<RegionModel> for &'a Region {
    fn into(self) -> RegionModel {
        use Region::*;
        match self {
            XA | Asia => RegionModel::XA,
            XC | PRC | PeoplesRepublicOfChina => RegionModel::XC,
            XE | Europe | EU => RegionModel::XE,
            XF | Africa => RegionModel::XF,
            XN | NorthAmerica | NAmerica => RegionModel::XN,
            XO | Oceania => RegionModel::XO,
            XQ | Antarctica => RegionModel::XQ,
            XS | SAmerica | SouthAmerica => RegionModel::XS,
            XU | UK | UnitedKingdom => RegionModel::XU,
            All => panic!("Region::All cannot be converted into seaplane::api::shared::v1::Region"),
        }
    }
}

/// A newtype wrapper to enforce where the ArgMatches came from which reduces
/// errors in checking if values of arguments were used or not. i.e. `seaplane
/// formation create` may not have the same arguments as `seaplane account
/// token` even though both produce an `ArgMatches`.
#[allow(missing_debug_implementations)]
#[derive(Debug)]
pub struct SeaplaneRestrictCommonArgMatches<'a>(pub &'a ArgMatches);

pub fn display_args() -> Vec<Arg> {
    vec![
        common::format(),
        arg!(--decode - ('D'))
            .help("Decode the directories before printing them")
            .long_help(LONG_DECODE)
            .overrides_with("no-decode"),
        arg!(--("no-decode"))
            .help("Print directories without decoding them")
            .overrides_with("decode"),
        arg!(--("no-header") | ("no-heading") | ("no-headers"))
            .help("Omit the header when printing with `--format=table`"),
    ]
}

pub fn restriction_details() -> Vec<Arg> {
    vec![
    arg!(--provider|providers =["PROVIDER"=>"all"]... ignore_case)
    .display_order(1)
    .next_line_help(true)
    .help("A provider where the data placement is allowed (supports comma separated list, or multiple uses)")
    .long_help(LONG_PROVIDER)
    .value_parser(value_parser!(Provider)),
    arg!(--("exclude-provider")|("exclude-providers") =["PROVIDER"]... ignore_case)
    .display_order(2)
    .next_line_help(true)
    .help("A provider where the data placement is *NOT* allowed (supports comma separated list, or multiple uses)")
    .long_help(LONG_EXCLUDE_PROVIDER)
    .value_parser(value_parser!(Provider)),
    arg!(--region|regions =["REGION"=>"all"]... ignore_case)
    .display_order(1)
    .next_line_help(true)
    .help("A region where the data placement is allowed (supports comma separated list, or multiple uses) (See REGION SPEC below)")
    .long_help(LONG_REGION)
    .value_parser(value_parser!(Region)),
    arg!(--("exclude-region")|("exclude-regions") =["REGION"]... ignore_case)
    .display_order(2)
    .next_line_help(true)
    .help("A region where the data placement is *NOT* allowed (supports comma separated list, or multiple uses) (See REGION SPEC below)")
    .long_help(LONG_EXCLUDE_REGION)
    .value_parser(value_parser!(Region)),
    ]
}

pub fn base64() -> Arg {
    arg!(--base64 - ('B')).help("The directory is already encoded in URL safe Base64")
}

pub fn api() -> Arg { arg!(api =["API"] required ).help("The API of the restricted directory") }

pub fn directory() -> Arg {
    arg!(directory =["DIRECTORY"] required ).help("The restricted directory")
}
