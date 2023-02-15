```console
$ seaplane formation list -h
List all local Formation Plans

Usage: seaplane[EXE] formation list [OPTIONS]

Options:
  -F, --fetch             Fetch remote Formation Instances and synchronize local DB prior to running this command [aliases: sync, synchronize]
  -v, --verbose...        Display more verbose output
      --format <FORMAT>   Change the output format [default: table] [possible values: table, json]
  -q, --quiet...          Suppress output at a specific level and below
      --color <COLOR>     Should the output include color? [default: auto] [possible values: always, ansi, auto, never]
      --no-color          Do not color output (alias for --color=never)
  -A, --api-key <STRING>  The API key associated with a Seaplane account used to access Seaplane API endpoints [env: SEAPLANE_API_KEY]
  -S, --stateless         Ignore local state files, do not read from or write to them
  -h, --help              Print help (see more with '--help')
  -V, --version           Print version

```

```console
$ seaplane formation list --help
List all local Formation Plans

This command will display the status each of your Formation Plans. The Formations displayed come
from the local database of known Formations. You may wish to update the local database with Remote
Formation Instances as well by either first running:

[..]

OR including `--fetch` such as:

[..]

After which your local database of Formations and Flights will contain all remote Formation
Instances as well.

Usage: seaplane[EXE] formation list [OPTIONS]

Options:
  -F, --fetch
          Fetch remote Formation Instances and synchronize local DB prior to running this command
          
          [aliases: sync, synchronize]

  -v, --verbose...
          Display more verbose output
          
          More uses displays more verbose output
              -v:  Display debug info
              -vv: Display trace info

      --format <FORMAT>
          Change the output format
          
          [default: table]
          [possible values: table, json]

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
