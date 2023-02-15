```console
$ seaplane formation land -h
Land a remote Formation Instance

Usage: seaplane[EXE] formation land [OPTIONS] <NAME|ID>

Arguments:
  <NAME|ID>  The name or ID of the Formation Instance to land

Options:
  -a, --all               Operate on all matching local Formation Plans even when the name or ID is ambiguous
  -v, --verbose...        Display more verbose output
  -F, --fetch             Fetch remote Formation Instances and synchronize local DB prior to running this command [aliases: sync, synchronize]
  -q, --quiet...          Suppress output at a specific level and below
      --color <COLOR>     Should the output include color? [default: auto] [possible values: always, ansi, auto, never]
      --no-color          Do not color output (alias for --color=never)
  -A, --api-key <STRING>  The API key associated with a Seaplane account used to access Seaplane API endpoints [env: SEAPLANE_API_KEY]
  -S, --stateless         Ignore local state files, do not read from or write to them
  -h, --help              Print help (see more with '--help')
  -V, --version           Print version

```

```console
$ seaplane formation land --help
Land (Stop) all configurations of a remote Formation Instance

Unlike 'seaplane formation delete' the land command does not delete the Formation from the local
database.

Usage: seaplane[EXE] formation land [OPTIONS] <NAME|ID>

Arguments:
  <NAME|ID>
          The name or ID of the Formation Instance to land

Options:
  -a, --all
          Operate on all matching local Formation Plans even when the name or ID is ambiguous

  -v, --verbose...
          Display more verbose output
          
          More uses displays more verbose output
              -v:  Display debug info
              -vv: Display trace info

  -F, --fetch
          Fetch remote Formation Instances and synchronize local DB prior to running this command
          
          [aliases: sync, synchronize]

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

      --no-color
          Do not color output (alias for --color=never)

  -A, --api-key <STRING>
          The API key associated with a Seaplane account used to access Seaplane API endpoints
          
          The value provided here will override any provided in any configuration files.
          A CLI provided value also overrides any environment variables.
          One can use a special value of '-' to signal the value should be read from STDIN.
          
          [env: SEAPLANE_API_KEY]

  -S, --stateless
          Ignore local state files, do not read from or write to them

  -h, --help
          Print help (see a summary with '-h')

  -V, --version
          Print version

```
