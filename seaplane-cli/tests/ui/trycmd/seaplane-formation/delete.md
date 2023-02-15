```console
$ seaplane formation delete -h
Deletes local Formation Plans and/or remote Formation Instances

Usage: 
    seaplane formation delete [OPTIONS] <NAME|ID>
    seaplane formation delete [OPTIONS] <NAME|ID> --no-remote

Arguments:
  <NAME|ID>  The name or ID of the Formation to remove, must be unambiguous

Options:
  -F, --fetch             Fetch remote Formation Instances and synchronize local DB prior to running this command [aliases: sync, synchronize]
  -v, --verbose...        Display more verbose output
  -a, --all               Operate on all matching local Formation Plans even when the name or ID is ambiguous
  -q, --quiet...          Suppress output at a specific level and below
      --color <COLOR>     Should the output include color? [default: auto] [possible values: always, ansi, auto, never]
  -f, --force             Delete this Formation even if there are remote instances without confirmation
      --local             Delete local Formation Definitions (this is set by the default, use --no-local to skip)
      --no-color          Do not color output (alias for --color=never)
  -A, --api-key <STRING>  The API key associated with a Seaplane account used to access Seaplane API endpoints [env: SEAPLANE_API_KEY]
      --no-local          DO NOT delete local Formation Definitions
      --remote            Delete remote Formation Instances (this is set by default, use --no-remote to skip)
  -S, --stateless         Ignore local state files, do not read from or write to them
      --no-remote         DO NOT delete remote Formation Instances (this is set by the default, use --remote to remove them)
  -h, --help              Print help (see more with '--help')
  -V, --version           Print version

```

```console
$ seaplane formation delete --help
Deletes local Formation Plans and/or remote Formation Instances

Usage: 
    seaplane formation delete [OPTIONS] <NAME|ID>
    seaplane formation delete [OPTIONS] <NAME|ID> --no-remote

Arguments:
  <NAME|ID>
          The name or ID of the Formation to remove, must be unambiguous

Options:
  -F, --fetch
          Fetch remote Formation Instances and synchronize local DB prior to running this command
          
          [aliases: sync, synchronize]

  -v, --verbose...
          Display more verbose output
          
          More uses displays more verbose output
              -v:  Display debug info
              -vv: Display trace info

  -a, --all
          Operate on all matching local Formation Plans even when the name or ID is ambiguous

  -q, --quiet...
          Suppress output at a specific level and below
          
          More uses suppresses higher levels of output
              -q:   Only display WARN messages and above
              -qq:  Only display ERROR messages
              -qqq: Suppress all output

      --color <COLOR>
          Should the output include color?
          
          [default: auto]
          [possible values: always, ansi, auto, never]

  -f, --force
          Delete this Formation even if there are remote instances without confirmation

      --local
          Delete local Formation Definitions (this is set by the default, use --no-local to skip)

      --no-color
          Do not color output (alias for --color=never)

  -A, --api-key <STRING>
          The API key associated with a Seaplane account used to access Seaplane API endpoints
          
          The value provided here will override any provided in any configuration files.
          A CLI provided value also overrides any environment variables.
          One can use a special value of '-' to signal the value should be read from STDIN.
          
          [env: SEAPLANE_API_KEY]

      --no-local
          DO NOT delete local Formation Definitions

      --remote
          Delete remote Formation Instances (this is set by default, use --no-remote to skip)

  -S, --stateless
          Ignore local state files, do not read from or write to them

      --no-remote
          DO NOT delete remote Formation Instances (this is set by the default, use --remote to remove them)

  -h, --help
          Print help (see a summary with '-h')

  -V, --version
          Print version

```
