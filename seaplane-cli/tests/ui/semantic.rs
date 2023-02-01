// The below tests are, "Do the CLI arguments we've set up have the semantics we expect"
//
// Additionally, we don't care about the output, just whether or not a run failed. These tests
// ensure as we change the CLI it maintains the same semantics
//
// Also note these runs don't actually do anything. They just parse the CLI so we don't need to
// mock anything or such.
//

macro_rules! cli {
    ($argv:expr) => {{
        match seaplane_cli::test_cli(const_format::concatcp!("seaplane ", $argv).split(" ")) {
            Ok(m) => seaplane_cli::test_main_update_ctx(&m),
            Err(e) => Err(e),
        }
    }};
}

#[test]
fn seaplane() {
    // The help is displayed
    assert!(cli!("").is_err());

    // For the OK tests we have to use a subcommand, so we pick init which was chosen by fair
    // diceroll.
    // --color and --no-color can override
    assert!(cli!("init --color=always --no-color").is_ok());
    // --quiet can stack
    assert!(cli!("init -qqq").is_ok());
    // --verbose can stack
    assert!(cli!("init -vvv").is_ok());
    // --api-key accepts '-' as a value
    assert!(cli!("init -A-").is_ok());
    // valid --color values
    assert!(cli!("init --color=always").is_ok());
    assert!(cli!("init --color=ansi").is_ok());
    assert!(cli!("init --color=auto").is_ok());
    assert!(cli!("init --color=never").is_ok());
    // --color values are not case sensitive
    assert!(cli!("init --color=Always").is_ok());
    assert!(cli!("init --color=ALWAYS").is_ok());
    assert!(cli!("init --color=AlWaYS").is_ok());
    // invalid --color values
    assert!(cli!("init --color=ishmael").is_err());
}

#[test]
fn seaplane_license() {
    assert!(cli!("license").is_ok());
    assert!(cli!("license --third-party").is_ok());
}

#[test]
fn seaplane_init() {
    assert!(cli!("init").is_ok());
    assert!(cli!("init --force").is_ok());
    // Force and overwrite can be used together
    assert!(cli!("init --force --overwrite=all").is_ok());

    // Valid overwrites
    assert!(cli!("init --overwrite=all").is_ok());
    assert!(cli!("init --overwrite=config").is_ok());
    assert!(cli!("init --overwrite=formations").is_ok());

    // Multiples
    assert!(cli!("init --overwrite=config,formations").is_ok());
    assert!(cli!("init --overwrite=config --overwrite=formations,all").is_ok());
    assert!(cli!("init --overwrite=config --overwrite=formations").is_ok());
    assert!(cli!("init --overwrite=config,all --overwrite=formations").is_ok());

    // Invalid overwrite
    assert!(cli!("init --overwrite=foo").is_err());
    // Invalid overwrite with --force is still error
    assert!(cli!("init --force --overwrite=foo").is_err());
}

#[test]
fn seaplane_account() {
    // help displayed
    let res = cli!("account");
    assert!(res.is_err(), "{res:?}");
}

#[test]
fn seaplane_account_token() {
    // The API key is required, but we manually check that and error if it's not present, so we
    // can't check it in the CLI tests

    // Give the API key
    assert!(cli!("account token -Afoo").is_ok());
    assert!(cli!("account -Afoo token").is_ok());
    assert!(cli!("-Afoo account token").is_ok());
}

#[test]
fn seaplane_account_login() {
    // API key required or it hangs so we can't test just the bare subcommand
    // Give the API key
    assert!(cli!("account login -Afoo").is_ok());
    assert!(cli!("account -Afoo login").is_ok());
    assert!(cli!("-Afoo account login").is_ok());
}

#[test]
fn seaplane_shell_completion() {
    // requires a SHELL
    assert!(cli!("shell-completion").is_err());
    // Invalid SHELL
    assert!(cli!("shell-completion bash").is_ok());
    // Give the SHELL
    assert!(cli!("shell-completion bash").is_ok());
    assert!(cli!("shell-completion zsh").is_ok());
    assert!(cli!("shell-completion powershell").is_ok());
    assert!(cli!("shell-completion elvish").is_ok());
    assert!(cli!("shell-completion fish").is_ok());
    // Shells are not case sensitive
    assert!(cli!("shell-completion Fish").is_ok());
    assert!(cli!("shell-completion FISH").is_ok());
    assert!(cli!("shell-completion fIsH").is_ok());
    // Invalid SHELL
    assert!(cli!("shell-completion jibberish").is_err());
}

#[test]
fn seaplane_flight() {
    // help is displayed
    assert!(cli!("flight").is_err());
}

#[test]
fn seaplane_flight_common() {
    // Because we use a common CLI set, we will use the copy command to test those common args
    // and then we don't need to re-test those args in each subcommand that simply reuses the
    // common arguments

    // aliases
    assert!(cli!("flight copy foo --maximum 2").is_ok());
    assert!(cli!("flight copy foo --max 2").is_ok());

    assert!(cli!("flight copy foo --minimum 2").is_ok());
    assert!(cli!("flight copy foo --min 2").is_ok());

    assert!(cli!("flight copy foo --api-permission").is_ok());
    assert!(cli!("flight copy foo --api-permissions").is_ok());

    assert!(cli!("flight copy foo --no-api-permission").is_ok());
    assert!(cli!("flight copy foo --no-api-permissions").is_ok());

    assert!(cli!("flight copy foo --architecture=amd64").is_ok());
    assert!(cli!("flight copy foo --architectures=amd64").is_ok());
    assert!(cli!("flight copy foo --arch=amd64").is_ok());
    assert!(cli!("flight copy foo --arches=amd64").is_ok());
    // --architecture case insensitive
    assert!(cli!("flight copy foo --arches=amd64").is_ok());
    assert!(cli!("flight copy foo --arches=AMD64").is_ok());
    assert!(cli!("flight copy foo --arches=AmD64").is_ok());

    // --architecture multiple items
    assert!(cli!("flight copy foo --arch=amd64,arm64").is_ok());
    assert!(cli!("flight copy foo --arch=amd64,arm64 --arch=amd64,arm64").is_ok());
    assert!(cli!("flight copy foo --arch=amd64 --arch=amd64").is_ok());
    assert!(cli!("flight copy foo --arch=amd64,arm64 --arch=amd64").is_ok());
    // cannot be multiple without comma or second use
    assert!(cli!("flight copy foo --arch amd64 arm64").is_err());

    // valid arches
    assert!(cli!("flight copy foo --arch=amd64,arm64").is_ok());
    // invalid arches
    assert!(cli!("flight copy foo --arch=pequod").is_err());

    // --no-* doesn't conflict
    assert!(cli!("flight copy foo --no-max --max=2").is_ok());
    assert!(cli!("flight copy foo --no-api-permission --api-permission").is_ok());
}

#[test]
fn seaplane_flight_copy() {
    // requires a NAME|ID
    assert!(cli!("flight copy").is_err());
    // provide a NAME|ID
    assert!(cli!("flight copy foo").is_ok());
    // invalid NAME|ID
    assert!(cli!("flight copy way-too-long-to-pass-validationway-too-loooong-to-pass-validation")
        .is_err());

    // clone is an alias
    assert!(cli!("flight clone foo").is_ok());
}

#[test]
fn seaplane_flight_create() {
    // provide an --image
    assert!(cli!("flight create --image ahab.com/alpine:latest").is_ok());
    assert!(cli!("flight create --image docker.io/library/nginx:latest --name foo").is_ok());
    // invalid name
    assert!(cli!(
        "flight --image ahab.com/alpine:latest --name create way-too-many-hyphens-to-pass-validation"
    )
    .is_err());

    // add is an alias
    assert!(cli!("flight add --image ahab.com/alpine:latest").is_ok());
}

#[test]
fn seaplane_flight_edit() {
    // requires a NAME|ID
    assert!(cli!("flight edit").is_err());
    // provide a NAME|ID
    assert!(cli!("flight edit foo").is_ok());
    // invalid NAME|ID
    assert!(cli!("flight edit way-too-long-to-pass-validationway-too-loooong-to-pass-validation")
        .is_err());
}

#[test]
fn seaplane_flight_delete() {
    // requires a NAME|ID
    assert!(cli!("flight delete").is_err());
    // provide a NAME|ID
    assert!(cli!("flight delete foo").is_ok());
    // invalid NAME|ID
    assert!(cli!(
        "flight delete way-too-long-to-pass-validationway-too-loooong-to-pass-validation"
    )
    .is_err());
    // --all and --exact conflict
    assert!(cli!("flight delete foo --all --exact").is_err());

    // aliases
    assert!(cli!("flight del foo").is_ok());
    assert!(cli!("flight remove foo").is_ok());
    assert!(cli!("flight rm foo").is_ok());
}

#[test]
fn seaplane_flight_list() {
    assert!(cli!("flight list").is_ok());

    // aliases
    assert!(cli!("flight ls").is_ok());
}

#[test]
fn seaplane_formation_common() {
    // Because we use a common CLI set, we will use the create command to test those common args
    // and then we don't need to re-test those args in each subcommand that simply reuses the
    // common arguments

    // Flight required
    assert!(cli!("formation plan").is_err());
    // valid name
    assert!(cli!("formation plan -I foo --name foo").is_ok());
    // invalid name
    assert!(cli!("formation plan -I foo --name way-too-many-hyphens-to-pass-validation").is_err());

    assert!(cli!("formation plan -I foo --launch").is_ok());
    assert!(cli!("formation plan -I foo --grounded").is_ok());
    // Same with it's alias
    assert!(cli!("formation plan -I foo --active").is_ok());
    assert!(cli!("formation plan -I foo --no-active").is_ok());
    // Overrides
    assert!(cli!("formation plan -I foo --launch --active").is_ok());
    // Should be good
    assert!(cli!("formation plan -I foo --launch --no-active").is_ok());
    assert!(cli!("formation plan -I foo --launch --grounded").is_ok());

    // flight
    // valid (@path requires a valid file...so we're not testing that and relying on the unit
    // tests for that functionality)
    assert!(cli!("formation plan --include-flight-plan @foo").is_ok());
    assert!(cli!("formation plan --include-flight-plan @-").is_ok());
    // invalid
    assert!(
        cli!("formation plan --include-flight-plan way-too-long-to-pass-validationway-too-loooong-to-pass-validation")
            .is_err()
    );
    // multiples only with commas or multiple uses
    assert!(cli!("formation plan --include-flight-plan=foo;bar;baz").is_ok());
    assert!(cli!("formation plan --include-flight-plan=foo -I=bar;baz").is_ok());
    assert!(cli!("formation plan --include-flight-plan=foo;bar -Ibaz").is_ok());
    assert!(cli!("formation plan --include-flight-plan foo bar baz").is_err());
    assert!(cli!("formation plan --include-flight-plan foo").is_ok());
    assert!(cli!("formation plan --include-flight-plan foo;bar").is_ok());
    assert!(cli!("formation plan --include-flight-plan foo;bar --include-flight-plan baz").is_ok());
    assert!(
        cli!("formation plan --include-flight-plan foo;bar --include-flight-plan baz;qux").is_ok()
    );
    assert!(cli!("formation plan --include-flight-plan foo --include-flight-plan baz;qux").is_ok());
    assert!(
        cli!("formation plan --include-flight-plan name=foo,image=demos.com/nginx:latest").is_ok()
    );
    assert!(cli!("formation plan --include-flight-plan foo;name=foo,image=demos.com/nginx:latest")
        .is_ok());
    assert!(cli!(
        "formation plan --include-flight-plan foo;name=bar,image=demos.com/nginx:latest;baz"
    )
    .is_ok());
    assert!(cli!("formation plan --include-flight-plan foo --include-flight-plan name=bar,image=demos.com/nginx:latest;baz").is_ok());
    assert!(cli!("formation plan --include-flight-plan foo;name=bar,image=demos.com/nginx:latest --include-flight-plan baz;qux").is_ok());
    assert!(cli!("formation plan --include-flight-plan name=bar,image=demos.com/nginx:latest;name=foo,image=demos.com/nginx:latest").is_ok());

    // affinity
    // valid
    assert!(cli!("formation plan -I foo --affinity foo").is_ok());
    // invalid
    assert!(
        cli!("formation plan -I foo --affinity way-too-many-hyphens-to-pass-validation").is_err()
    );
    // multiples only with commas or multiple uses
    assert!(cli!("formation plan -I foo --affinity=foo,bar,baz").is_ok());
    assert!(cli!("formation plan -I foo --affinity=foo --affinity=bar,baz").is_ok());
    assert!(cli!("formation plan -I foo --affinity=foo,bar --affinity=baz").is_ok());
    assert!(cli!("formation plan -I foo --affinity foo bar baz").is_err());
    // alias
    assert!(cli!("formation plan -I foo --affinities foo").is_ok());

    // connection
    // valid
    assert!(cli!("formation plan -I foo --connection foo").is_ok());
    // invalid
    assert!(
        cli!("formation plan -I foo --connection way-too-many-hyphens-to-pass-validation").is_err()
    );
    // multiples only with commas or multiple uses
    assert!(cli!("formation plan -I foo --connection=foo,bar,baz").is_ok());
    assert!(cli!("formation plan -I foo --connection=foo --connection=bar,baz").is_ok());
    assert!(cli!("formation plan -I foo --connection=foo,bar --connection=baz").is_ok());
    assert!(cli!("formation plan -I foo --connection foo bar baz").is_err());
    // alias
    assert!(cli!("formation plan -I foo --connections foo").is_ok());

    // provider
    // valid
    assert!(cli!("formation plan -I foo --provider=Aws,Azure,DigitalOcean,Equinix,Gcp,All").is_ok());
    // invalid
    assert!(cli!("formation plan -I foo --provider=carpet").is_err());
    // Case insensitive
    assert!(cli!("formation plan -I foo --provider=AWS").is_ok());
    assert!(cli!("formation plan -I foo --provider=aws").is_ok());
    assert!(cli!("formation plan -I foo --provider=aWs").is_ok());
    // multiples only with commas or multiple uses
    assert!(cli!("formation plan -I foo --provider=Aws,Azure --provider=DigitalOcean").is_ok());
    assert!(cli!("formation plan -I foo --provider=Aws --provider=DigitalOcean,Azure").is_ok());
    assert!(cli!("formation plan -I foo --provider=Aws --provider=DigitalOcean").is_ok());
    assert!(cli!("formation plan -I foo --provider Aws Azure DigitalOcean").is_err());
    // alias
    assert!(
        cli!("formation plan -I foo --providers=Aws,Azure,DigitalOcean,Equinix,Gcp,All").is_ok()
    );

    // exclude provider
    // valid
    assert!(cli!(
        "formation plan -I foo --exclude-provider=Aws,Azure,DigitalOcean,Equinix,Gcp,All"
    )
    .is_ok());
    // invalid
    assert!(cli!("formation plan -I foo --exclude-provider=carpet").is_err());
    // Case insensitive
    assert!(cli!("formation plan -I foo --exclude-provider=AWS").is_ok());
    assert!(cli!("formation plan -I foo --exclude-provider=aws").is_ok());
    assert!(cli!("formation plan -I foo --exclude-provider=aWs").is_ok());
    // multiples only with commas or multiple uses
    assert!(cli!(
        "formation plan -I foo --exclude-provider=Aws,Azure --exclude-provider=DigitalOcean"
    )
    .is_ok());
    assert!(cli!(
        "formation plan -I foo --exclude-provider=Aws --exclude-provider=DigitalOcean,Azure"
    )
    .is_ok());
    assert!(cli!("formation plan -I foo --exclude-provider=Aws --exclude-provider=DigitalOcean")
        .is_ok());
    assert!(cli!("formation plan -I foo --exclude-provider Aws Azure DigitalOcean").is_err());
    // alias
    assert!(cli!(
        "formation plan -I foo --exclude-providers=Aws,Azure,DigitalOcean,Equinix,Gcp,All"
    )
    .is_ok());

    // region
    // valid
    assert!(
        cli!("formation plan -I foo --region=XA,Asia,XC,PRC,PeoplesRepublicofChina,XE,Europe,EU,XF,Africa,XN,NorthAmerica,NAmerica,XO,Oceania,XQ,Antarctica,XS,SAmerica,SouthAmerica,XU,UK,UnitedKingdom,All")
            .is_ok()
    );
    // invalid
    assert!(cli!("formation plan -I foo --region=carpet").is_err());
    // Case insensitive
    assert!(cli!("formation plan -I foo --region=Oceania").is_ok());
    assert!(cli!("formation plan -I foo --region=oceania").is_ok());
    assert!(cli!("formation plan -I foo --region=OcEanIa").is_ok());
    // multiples only with commas or multiple uses
    assert!(cli!("formation plan -I foo --region=XA,prc --region=europe").is_ok());
    assert!(cli!("formation plan -I foo --region=eu --region=xn,xs").is_ok());
    assert!(cli!("formation plan -I foo --region=uk --region=namerica").is_ok());
    assert!(cli!("formation plan -I foo --region xa xc xf").is_err());
    // alias
    assert!(cli!("formation plan -I foo --regions=XA,Asia,XC").is_ok());

    // exclude region
    // valid
    assert!(
        cli!("formation plan -I foo --exclude-region=XA,Asia,XC,PRC,PeoplesRepublicofChina,XE,Europe,EU,XF,Africa,XN,NorthAmerica,NAmerica,XO,Oceania,XQ,Antarctica,XS,SAmerica,SouthAmerica,XU,UK,UnitedKingdom,All")
            .is_ok()
    );
    // invalid
    assert!(cli!("formation plan -I foo --exclude-region=carpet").is_err());
    // Case insensitive
    assert!(cli!("formation plan -I foo --exclude-region=Oceania").is_ok());
    assert!(cli!("formation plan -I foo --exclude-region=oceania").is_ok());
    assert!(cli!("formation plan -I foo --exclude-region=OcEanIa").is_ok());
    // multiples only with commas or multiple uses
    assert!(cli!("formation plan -I foo --exclude-region=XA,prc --exclude-region=europe").is_ok());
    assert!(cli!("formation plan -I foo --exclude-region=eu --exclude-region=xn,xs").is_ok());
    assert!(cli!("formation plan -I foo --exclude-region=uk --exclude-region=namerica").is_ok());
    assert!(cli!("formation plan -I foo --exclude-region xa xc xf").is_err());
    // alias
    assert!(cli!("formation plan -I foo --exclude-regions=XA,Asia,XC").is_ok());

    // public endpoint (we don't try to enumerate valid endpoints because that should happen
    // in unit tests, just that *some* form of validation happens along with any semantics such
    // as multiples, etc.)
    // valid
    assert!(cli!("formation plan -I foo --public-endpoint=http:/foo/bar=baz:123").is_ok());
    assert!(cli!("formation plan -I foo --public-endpoint=/foo/bar=baz:123").is_ok());
    // invalid
    assert!(cli!("formation plan -I foo --public-endpoint=carpet").is_err());
    assert!(cli!("formation plan -I foo --public-endpoint=http:foo/bar=baz:123").is_err());
    assert!(cli!("formation plan -I foo --public-endpoint=foo/bar=baz:123").is_err());
    // multiples only with commas or multiple uses
    assert!( cli!("formation plan -I foo --public-endpoint=http:/foo/bar=baz:123,/baz/qux=nom:432 --public-endpoint=http:/=que:5432") .is_ok());
    assert!( cli!("formation plan -I foo --public-endpoint=https:/foo/bar=baz:123 --public-endpoint /=que:5432,http:/baz/qux=nom:432") .is_ok());
    assert!(cli!(
        "formation plan -I foo --public-endpoint=http:/foo/bar=baz:123 --public-endpoint /=que:5432"
    )
    .is_ok());
    assert!( cli!("formation plan -I foo --public-endpoint=http:foo/bar=baz:123 http:baz/qux=nom:432 http:/=que:5432") .is_err());
    // alias
    assert!(cli!("formation plan -I foo --public-endpoints /foo/bar=baz:123").is_ok());

    // formation endpoint (we don't try to enumerate valid endpoints because that should happen
    // in unit tests, just that *some* form of validation happens along with any semantics such
    // as multiples, etc.)
    // valid
    assert!(cli!("formation plan -I foo --formation-endpoint=tcp:22=baz:123").is_ok());
    // invalid
    assert!(cli!("formation plan -I foo --formation-endpoint=carpet").is_err());
    // multiples only with commas or multiple uses
    assert!( cli!("formation plan -I foo --formation-endpoint=http:/foo/bar=baz:123,udp:987=nom:432 --formation-endpoint=http:/=que:5432") .is_ok());
    assert!( cli!("formation plan -I foo --formation-endpoint=tcp:123=baz:123 --formation-endpoint /=que:5432,udp:876=nom:432") .is_ok());
    assert!(cli!(
        "formation plan -I foo --formation-endpoint=udp:1234=baz:123 --formation-endpoint /=que:5432"
    )
    .is_ok());
    assert!( cli!("formation plan -I foo --formation-endpoint=http:/foo/bar=baz:123 tcp:baz/qux=nom:432 http:/=que:5432") .is_err());
    // alias
    assert!(cli!("formation plan -I foo --formation-endpoints=udp:1234=baz:123").is_ok());

    // flight endpoint (we don't try to enumerate valid endpoints because that should happen
    // in unit tests, just that *some* form of validation happens along with any semantics such
    // as multiples, etc.)
    // valid
    assert!(cli!("formation plan -I foo --flight-endpoint=tcp:22=baz:123").is_ok());
    // invalid
    assert!(cli!("formation plan -I foo --flight-endpoint=carpet").is_err());
    // multiples only with commas or multiple uses
    assert!( cli!("formation plan -I foo --flight-endpoint=http:/foo/bar=baz:123,udp:987=nom:432 --flight-endpoint=http:/=que:5432") .is_ok());
    assert!( cli!("formation plan -I foo --flight-endpoint=tcp:123=baz:123 --flight-endpoint=http:/=que:5432,udp:876=nom:432") .is_ok());
    assert!(cli!(
        "formation plan -I foo --flight-endpoint=udp:1234=baz:123 --flight-endpoint=http:/=que:5432"
    )
    .is_ok());
    assert!( cli!("formation plan -I foo --flight-endpoint=http:/foo/bar=baz:123 tcp:baz/qux=nom:432 http:/=que:5432") .is_err());
    // alias
    assert!(cli!("formation plan -I foo --flight-endpoints=udp:1234=baz:123").is_ok());
}

#[test]
fn seaplane_formation_list() {
    assert!(cli!("formation list").is_ok());

    // aliases
    assert!(cli!("formation ls").is_ok());
}

#[test]
fn seaplane_formation_plan() {
    // requires flight
    assert!(cli!("formation plan").is_err());
    // invalid name
    assert!(cli!("formation -I foo --name plan way-too-many-hyphens-to-pass-validation").is_err());

    // options
    assert!(cli!("formation plan -I foo --force").is_ok());
    assert!(cli!("formation plan -I foo --launch").is_ok());
    assert!(cli!("formation plan -I foo --grounded").is_ok());
    // overrides
    assert!(cli!("formation plan -I foo --launch --active").is_ok());
    // should be OK but not override
    assert!(cli!("formation plan -I foo --launch --no-active").is_ok());

    // add is an alias
    assert!(cli!("formation add -I foo").is_ok());
    assert!(cli!("formation create -I foo").is_ok());
}

#[test]
fn seaplane_formation_delete() {
    // requires a NAME|ID
    assert!(cli!("formation delete").is_err());
    // provide a NAME|ID
    assert!(cli!("formation delete foo").is_ok());
    // invalid NAME|ID
    assert!(cli!("formation delete way-too-many-hyphens-to-pass-validation").is_err());
    assert!(cli!("formation delete foo --remote").is_ok());
    assert!(cli!("formation delete foo --local").is_ok());
    assert!(cli!("formation delete foo --no-remote").is_ok());
    assert!(cli!("formation delete foo --no-local").is_ok());
    assert!(cli!("formation delete foo --remote --no-remote").is_ok());
    assert!(cli!("formation delete foo --local --no-local").is_ok());
    // --all and --exact conflict
    assert!(cli!("formation delete foo --all --exact").is_err());

    // aliases
    assert!(cli!("formation del foo").is_ok());
    assert!(cli!("formation remove foo").is_ok());
    assert!(cli!("formation rm foo").is_ok());
}

#[test]
fn seaplane_formation_fetch_remote() {
    assert!(cli!("formation fetch-remote").is_ok());
    assert!(cli!("formation fetch-remote foo").is_ok());

    // aliases
    assert!(cli!("formation fetch").is_ok());
}

#[test]
fn seaplane_formation_launch() {
    // requires a NAME|ID
    assert!(cli!("formation launch").is_err());
    // provide a NAME|ID
    assert!(cli!("formation launch foo").is_ok());
    // invalid NAME|ID
    assert!(cli!("formation launch way-too-many-hyphens-to-pass-validation").is_err());
    assert!(cli!("formation launch foo --fetch").is_ok());
    assert!(cli!("formation launch foo --grounded").is_ok());
    // --all and --exact conflict
    assert!(cli!("formation launch foo --all --exact").is_err());

    // aliases
    assert!(cli!("formation start foo").is_ok());
}

#[test]
fn seaplane_formation_land() {
    // requires a NAME|ID
    assert!(cli!("formation land").is_err());
    // provide a NAME|ID
    assert!(cli!("formation land foo").is_ok());
    // invalid NAME|ID
    assert!(cli!("formation land way-too-many-hyphens-to-pass-validation").is_err());
    // --all and --exact conflict
    assert!(cli!("formation land foo --all --exact").is_err());

    // aliases
    assert!(cli!("formation stop foo").is_ok());
}

#[test]
fn seaplane_md() {
    // requires a subcmd
    assert!(cli!("metadata").is_err());
    // provide subcmd
    assert!(cli!("metadata delete foo").is_ok());

    // aliases
    assert!(cli!("md delete foo").is_ok());
    assert!(cli!("meta delete foo").is_ok());
}

#[test]
fn seaplane_md_delete() {
    // requires a KEY
    assert!(cli!("metadata delete").is_err());
    // provide a key
    assert!(cli!("metadata delete foo").is_ok());
    // multiples
    assert!(cli!("metadata delete foo bar baz").is_ok());
    assert!(cli!("metadata delete foo,bar,baz").is_ok());
    assert!(cli!("metadata delete foo bar,baz").is_ok());
    assert!(cli!("metadata delete foo,bar baz").is_ok());

    // aliases
    assert!(cli!("metadata del foo").is_ok());
    assert!(cli!("metadata remove foo").is_ok());
    assert!(cli!("metadata rm foo").is_ok());
}

#[test]
fn seaplane_md_get() {
    // requires a KEY
    assert!(cli!("metadata get").is_err());
    // provide a key
    assert!(cli!("metadata get foo").is_ok());
    // can not have multiples
    assert!(cli!("metadata get foo bar").is_err());
    assert!(cli!("metadata get foo bar baz").is_err());
    // comma is not a value delimiter
    assert!(cli!("metadata get foo,bar,baz").is_ok());
    assert!(cli!("metadata get foo bar,baz").is_err());
    assert!(cli!("metadata get foo,bar baz").is_err());

    // aliases
    assert!(cli!("metadata show foo").is_ok());

    // can't have both --only-keys and --only-values
    assert!(cli!("metadata get foo --only-keys --only-values").is_err());
}

#[test]
fn seaplane_md_set() {
    // requires a KEY and VALUE
    assert!(cli!("metadata set").is_err());
    assert!(cli!("metadata set foo").is_err());
    // provide a valid KEY VALUE
    assert!(cli!("metadata set foo bar").is_ok());
    // multiples are not allowed
    assert!(cli!("metadata set foo bar baz qux").is_err());
    // comma is not a value delimiter
    assert!(cli!("metadata set foo,bar").is_err());
    assert!(cli!("metadata set foo bar,baz").is_ok());
    assert!(cli!("metadata set foo,bar baz").is_ok());

    // aliases
    assert!(cli!("metadata put foo bar").is_ok());
}

#[test]
fn seaplane_md_list() {
    // does not require a dir
    assert!(cli!("metadata list").is_ok());
    // can provide a dir
    assert!(cli!("metadata list foo").is_ok());
    // multiples not supported
    assert!(cli!("metadata list foo bar").is_err());
    assert!(cli!("metadata list foo bar baz").is_err());
    // comma is not a value delimiter
    assert!(cli!("metadata list foo,bar,baz").is_ok());
    assert!(cli!("metadata list foo bar,baz").is_err());
    assert!(cli!("metadata list foo,bar baz").is_err());

    // aliases
    assert!(cli!("metadata ls foo").is_ok());

    // can't have both --only-keys and --only-values
    assert!(cli!("metadata list --only-keys --only-values").is_err());
}

#[test]
fn seaplane_locks() {
    // requires a subcmd
    assert!(cli!("locks").is_err());
    // provide subcmd
    assert!(cli!("locks list foo").is_ok());
}

#[test]
fn seaplane_locks_release() {
    // requires a LOCK_NAME and LOCK_ID
    assert!(cli!("locks release").is_err());
    assert!(cli!("locks release foo").is_err());
    assert!(cli!("locks release --lock-id bar").is_err());
    // provide LOCK_NAME, LOCK_ID
    assert!(cli!("locks release foo --lock-id bar").is_ok());
    // can not have multiples
    assert!(cli!("locks release foo baz --lock-id bar").is_err());
    assert!(cli!("locks release foo --lock-id bar baz").is_err());

    // aliases
    assert!(cli!("locks rl foo --lock-id bar").is_ok());
}

#[test]
fn seaplane_locks_list() {
    // list all locks if LOCK_NAME is omitted
    assert!(cli!("locks list").is_ok());
    // provide a LOCK_NAME
    assert!(cli!("locks list foo").is_ok());
    // can not have multiples
    assert!(cli!("locks list foo bar").is_err());
    assert!(cli!("locks list foo bar baz").is_err());
    // comma is not a value delimiter
    assert!(cli!("locks list foo,bar,baz").is_ok());
    assert!(cli!("locks list foo bar,baz").is_err());
    assert!(cli!("locks list foo,bar baz").is_err());

    // aliases
    assert!(cli!("locks ls foo").is_ok());
}

#[test]
fn seaplane_locks_renew() {
    // requires a LOCK_NAME and LOCK_ID and TTL
    assert!(cli!("locks renew").is_err());
    assert!(cli!("locks renew foo").is_err());
    assert!(cli!("locks renew foo --lock-id bar").is_err());
    assert!(cli!("locks renew foo --ttl 30").is_err());
    assert!(cli!("locks renew --lock-id bar --ttl 30").is_err());
    // provide valid LOCK_NAME, LOCK_ID and TTL
    assert!(cli!("locks renew foo --lock-id bar --ttl 30").is_ok());
    // multiples are not allowed
    assert!(cli!("locks renew foo baz --lock-id bar --ttl 30").is_err());
    assert!(cli!("locks renew foo baz qux --lock-id bar --ttl 30").is_err());
    assert!(cli!("locks renew foo, baz, qux --lock-id bar --ttl 30").is_err());
    assert!(cli!("locks renew foo baz, qux --lock-id bar --ttl 30").is_err());
    assert!(cli!("locks renew foo, baz qux --lock-id bar --ttl 30").is_err());
    assert!(cli!("locks renew foo --lock-id bar baz --ttl 30").is_err());
    assert!(cli!("locks renew foo --lock-id bar, baz --ttl 30").is_err());
    assert!(cli!("locks renew foo --lock-id bar --ttl 30 60").is_err());
    assert!(cli!("locks renew foo --lock-id bar --ttl 30, 60").is_err());
}

#[test]
fn seaplane_locks_acquire() {
    // requires a LOCK_NAME and CLIENT_ID and TTL
    assert!(cli!("locks acquire").is_err());
    assert!(cli!("locks acquire foo").is_err());
    assert!(cli!("locks acquire foo --client-id bar").is_err());
    assert!(cli!("locks acquire foo --ttl 60").is_err());
    assert!(cli!("locks acquire --client-id bar --ttl 60").is_err());
    assert!(cli!("locks acquire --client-id bar").is_err());
    assert!(cli!("locks acquire --ttl 60").is_err());
    // provide LOCK_NAME, CLIENT_ID, TTL
    assert!(cli!("locks acquire foo --client-id bar --ttl 60").is_ok());
    // can not have multiples
    assert!(cli!("locks acquire foo bar").is_err());
    assert!(cli!("locks acquire foo baz --client-id bar --ttl 60").is_err());
    assert!(cli!("locks acquire foo, baz --client-id bar --ttl 60").is_err());
    assert!(cli!("locks acquire foo --client-id bar baz --ttl 60").is_err());
    assert!(cli!("locks acquire foo --client-id bar, baz --ttl 60").is_err());
    assert!(cli!("locks acquire foo --client-id bar --ttl 60 30").is_err());
    assert!(cli!("locks acquire foo --client-id bar --ttl 60, 30").is_err());

    // aliases
    assert!(cli!("locks acq foo --client-id bar --ttl 60").is_ok());
}

#[test]
fn seaplane_restrict() {
    // requires a subcmd
    assert!(cli!("restrict").is_err());
    // provide subcmd
    assert!(cli!("restrict get config foo/bar").is_ok());
}

#[test]
fn seaplane_restrict_get() {
    // requires API and directory
    assert!(cli!("restrict get").is_err());
    assert!(cli!("restrict get config").is_err());
    assert!(cli!("restrict get foo/bar").is_err());

    // provide API and directory
    assert!(cli!("restrict get config foo").is_ok());

    // three is a crowd
    assert!(cli!("restrict get foo bar baz").is_err());
}

#[test]
fn seaplane_restrict_list() {
    // requires no args or just API
    assert!(cli!("restrict list").is_ok());
    assert!(cli!("restrict list config").is_ok());
    assert!(cli!("restrict list config foo/bar").is_err());

    assert!(cli!("restrict list config -D").is_ok());

    assert!(cli!("restrict list config --unknown_option").is_err());
}

#[test]
fn seaplane_restrict_delete() {
    // requires API and directory
    assert!(cli!("restrict delete").is_err());
    assert!(cli!("restrict delete config").is_err());
    assert!(cli!("restrict delete foo/bar").is_err());

    // provide API and directory
    assert!(cli!("restrict delete config foo").is_ok());

    // three is a crowd
    assert!(cli!("restrict delete foo bar baz").is_err());
}

#[test]
fn seaplane_restrict_set() {
    // requires API, directory and restriction details
    assert!(cli!("restrict set").is_err());
    assert!(cli!("restrict set config").is_err());
    assert!(cli!("restrict set foo/bar").is_err());

    // too many arguments
    assert!(cli!("restrict set foo bar baz").is_err());

    // wrong option
    assert!(cli!("restrict set config foo --unknown-option").is_err());

    // wrong option usage
    assert!(cli!("restrict set config foo --provider not_a_provider").is_err());
    assert!(cli!("restrict set config foo --region not_a_region").is_err());

    // some happy paths
    assert!(cli!("restrict set config foo --provider aws").is_ok());
    assert!(cli!("restrict set config foo --exclude-provider azure").is_ok());
    assert!(cli!("restrict set config foo --provider aws --exclude-provider azure").is_ok());
    assert!(cli!("restrict set config foo --provider aws --exclude-provider azure --region xe --exclude-region xn").is_ok());
    assert!(cli!("restrict set config foo --region xe").is_ok());
    assert!(cli!("restrict set config foo --exclude-region xn").is_ok());
    assert!(cli!("restrict set config foo --region xe --exclude-region xn").is_ok());
    assert!(cli!("restrict set config foo --region EuRoPe --exclude-region Namerica").is_ok());

    // lists everywhere
    assert!(cli!("restrict set config foo --provider aws,digitalocean --exclude-provider azure,gcp --region xe,xs --exclude-region xn,xc").is_ok());

    // default is all providers and regions allowed
    assert!(cli!("restrict set config foo").is_ok());
}
