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
fn seaplane_formation_list() {
    assert!(cli!("formation list").is_ok());

    // aliases
    assert!(cli!("formation ls").is_ok());

    // fetch and its aliases (here we use the list command just to make it less confusing)
    assert!(cli!("formation list --fetch").is_ok());
    assert!(cli!("formation list --sync").is_ok());
    assert!(cli!("formation list --synchronize").is_ok());
    assert!(cli!("formation list -F").is_ok());
}

// Note: PATH validity checks are skipped in UI tests and no Flight is loaded
#[test]
fn seaplane_formation_plan() {
    // requires flight
    assert!(cli!("formation plan").is_err());
    // invalid name
    assert!(
        cli!("formation plan -F foo.json --name way-too-many-hyphens-to-pass-validation").is_err()
    );

    // options
    let res = cli!("formation plan -F foo.json --force");
    assert!(res.is_ok(), "{res:?}");
    assert!(cli!("formation plan -F foo.json --launch").is_ok());

    // add is an alias
    assert!(cli!("formation add -F foo.json").is_ok());
    assert!(cli!("formation create -F foo.json").is_ok());

    // flight
    assert!(cli!("formation plan --flight foo.json").is_ok());
    let res = cli!("formation plan --flight -");
    assert!(res.is_ok(), "{res:?}");
    assert!(cli!("formation plan -F-").is_ok());
    // multiples
    // Technically VAL2;VAL2 is the correct way to pass two flights, however when _not_ using the
    // INLINE flight spec people probably expect VAL1,VAL2. So the way our validation works we
    // should "fail over" to the correct thing and all these combinations should work fine.
    //
    // i.e. these should be semantically the same, `foo.json,bar.json` and `foo.json;bar.json`.
    // Additionally, an inline flight spec should be able to be mixed in as well, so
    // `name=foo,image=bar;baz.json` should be fine.
    assert!(cli!("formation plan --flight=foo.json,bar.json,baz.json").is_ok());
    assert!(cli!("formation plan --flight=foo.json -F=bar.json,baz.json").is_ok());
    assert!(cli!("formation plan --flight=foo.json,bar.json -Fbaz.json").is_ok());
    assert!(cli!("formation plan --flight foo.json bar.json baz.json").is_err());
    assert!(cli!("formation plan --flight foo.json").is_ok());
    assert!(cli!("formation plan --flight foo.json;bar.json").is_ok());
    assert!(cli!("formation plan --flight foo.json;bar.json --flight baz.json").is_ok());
    assert!(cli!("formation plan --flight foo.json,bar.json --flight baz.json;qux.json").is_ok());
    assert!(cli!("formation plan --flight foo.json --flight baz.json;qux.json").is_ok());
    assert!(cli!("formation plan --flight name=foo,image=demos.com/nginx:latest").is_ok());
    assert!(cli!("formation plan --flight foo.json;name=foo,image=demos.com/nginx:latest").is_ok());
    assert!(cli!(
        "formation plan --flight foo.json;name=bar,image=demos.com/nginx:latest;baz.json"
    )
    .is_ok());
    assert!(cli!(
        "formation plan --flight foo.json --flight name=bar,image=demos.com/nginx:latest;baz.json"
    )
    .is_ok());
    assert!(cli!(
        "formation plan --flight foo;name=bar,image=demos.com/nginx:latest --flight baz.json;qux.json"
    )
    .is_ok());
    assert!(cli!("formation plan --flight name=bar,image=demos.com/nginx:latest;name=foo,image=demos.com/nginx:latest").is_ok());
    // WARNING: The one thing that should fail is mixing inline specs and paths with only a comma
    // such as `name=foo,image=bar,baz.json`
    assert!(cli!("formation plan -F name=bar,image=demos.com/nginx:latest,baz.json").is_err());

    // Gatweay Flight
    // valid
    assert!(cli!("formation plan -F foo.json, --gateway-flight=foo").is_ok());
    assert!(cli!("formation plan -F foo.json, -G=foo").is_ok());
    // invalid
    assert!(cli!("formation plan -F foo.json --gateway-flight=baz --gateway-flight=que").is_err());
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
    // valid name/ID (all IDs are valid names, but only some names are valid IDs)
    assert!(cli!("formation fetch-remote").is_ok());
    assert!(cli!("formation fetch-remote foo").is_ok());
    assert!(cli!("formation fetch-remote frm-5wacbutjwbdexonddvdb2lnyxu").is_ok());
    // invalid name/ID
    assert!(cli!("formation fetch-remote way-too-many-hyphens-to-pass-validation").is_err());

    // aliases
    assert!(cli!("formation fetch").is_ok());
}

#[test]
fn seaplane_formation_launch() {
    // requires a NAME
    assert!(cli!("formation launch").is_err());
    // provide a NAME
    assert!(cli!("formation launch foo").is_ok());
    // invalid NAME
    assert!(cli!("formation launch way-too-many-hyphens-to-pass-validation").is_err());
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
