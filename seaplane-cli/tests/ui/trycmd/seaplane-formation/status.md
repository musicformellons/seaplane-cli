```console
$ seaplane formation status -h
Show the status of a remote Formation Instance

Usage: seaplane[EXE] formation status [OPTIONS] [NAME|ID]

Arguments:
  [NAME|ID]  The name or ID of the Formation to check, must be unambiguous

Options:
      --format <FORMAT>   Change the output format [default: table] [possible values: table, json]
  -v, --verbose...        Display more verbose output
      --no-fetch          Skip fetching and synchronizing of remote instances
  -q, --quiet...          Suppress output at a specific level and below
      --color <COLOR>     Should the output include color? [default: auto] [possible values: always, ansi, auto, never]
      --no-color          Do not color output (alias for --color=never)
  -A, --api-key <STRING>  The API key associated with a Seaplane account used to access Seaplane API endpoints [env: SEAPLANE_API_KEY]
  -S, --stateless         Ignore local state files, do not read from or write to them
  -h, --help              Print help (see more with '--help')
  -V, --version           Print version

```

```console
$ seaplane formation status --help
Show the status of a remote Formation Instance

This command will display the status of one or more Formation Instances such as how many actual
containers are running compared to the minimum and maximums per Flight that the configuration
defines.

Usage: seaplane[EXE] formation status [OPTIONS] [NAME|ID]

Arguments:
  [NAME|ID]
          The name or ID of the Formation to check, must be unambiguous

Options:
      --format <FORMAT>
          Change the output format
          
          [default: table]
          [possible values: table, json]

  -v, --verbose...
          Display more verbose output
          
          More uses displays more verbose output
              -v:  Display debug info
              -vv: Display trace info

      --no-fetch
          Skip fetching and synchronizing of remote instances

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
