The short help message with `-h`:

```console
$ seaplane formation plan -h
Create a Seaplane Formation

Usage: seaplane[EXE] formation plan [OPTIONS] --flight <SPEC>

Options:
      --fetch                  Fetch remote Formation Instances and synchronize local DB prior to running this command [aliases: sync, synchronize]
  -v, --verbose...             Display more verbose output
  -n, --name <STRING>          A friendly name for the Formation (unique within the tenant) if omitted a pseudo random name will be assigned. Note the name appears as part of the public Formation URL
  -q, --quiet...               Suppress output at a specific level and below
      --color <COLOR>          Should the output include color? [default: auto] [possible values: always, ansi, auto, never]
      --launch                 This Formation Plan should be deployed right away
  -F, --flight <SPEC>          Use Flight in this Formation in the form of SPEC|path|- (supports semicolon (';') separated list, or multiple uses) (See FLIGHT SPEC below) [aliases: flights]
      --no-color               Do not color output (alias for --color=never)
  -A, --api-key <STRING>       The API key associated with a Seaplane account used to access Seaplane API endpoints [env: SEAPLANE_API_KEY]
  -G, --gateway-flight <NAME>  The name of the Flight to be used as the public traffic gateway that will receive all traffic that arrives on the public URL (if only a single Flight is included in this Formation, it will be implied as the gateway)
      --force                  Override any existing Formation with the same NAME
  -S, --stateless              Ignore local state files, do not read from or write to them
  -h, --help                   Print help (see more with '--help')
  -V, --version                Print version

FLIGHT SPEC

    The Flight may be specified in one of the following ways

    FLIGHT_SPEC := INLINE-SPEC | PATH | -
    PATH        := PATH is an existing file with a Flight in JSON format
    -           := STDIN will be read for a Flight in JSON format
    INLINE-SPEC := Comma separated LIST of ATTRIBUTE
    ATTRIBUTE   := image=IMAGE [ | name=NAME | minimum=NUM | maximum=NUM | api-permission | architecture=ARCH ]
    NUM         := Positive integer (minimum default is 1 if omitted; maximum default is 'autoscale as needed')
    ARCH        := amd64 | arm64

    NOTE that when using - only one Flight may be provided via STDIN

```

The long help message with `--help`:

```console
$ seaplane formation plan --help
Make a new local Formation Plan (and optionally launch an instance of it)

Include Flights by using `--flight`. Multiple Flights may be included in a Formation Plan using a
SEMICOLON separated list, or using the argument multiple times.

Usage: seaplane[EXE] formation plan [OPTIONS] --flight <SPEC>

Options:
      --fetch
          Fetch remote Formation Instances and synchronize local DB prior to running this command
          
          [aliases: sync, synchronize]

  -v, --verbose...
          Display more verbose output
          
          More uses displays more verbose output
              -v:  Display debug info
              -vv: Display trace info

  -n, --name <STRING>
          A human readable name for the Formation (must be unique within the tenant)
          
          Rules for a valid name are as follows:
          
            - may only include ASCII lowercase, numbers and hyphens (0-9, a-z, and '-')
            - hyphens ('-') may not be repeated (i.e. '--')
            - no more than three (3) total hyphens
            - may not start or end with a hyphen
            - the total length must be <= 63
          
          Some of these restrictions may be lifted in the future.

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

      --launch
          This Formation Plan should be deployed right away

  -F, --flight <SPEC>
          A Flight to include in this Formation in the form of SPEC|path|- (See FLIGHT SPEC below)
          
          Multiple items can be passed as a SEMICOLON (';') separated list or by using the argument multiple
          times. Note that when using the SPEC it's usually easiest to only place one Flight per --flight
          argument.
          
          $ seaplane formation plan /
              --flight name=flight1,image=nginx:latest /
              --flight name=flight2,image=hello:latest
          
          Which would include, two Flights (flight1, and flight2).
          
          [aliases: flights]

      --no-color
          Do not color output (alias for --color=never)

  -A, --api-key <STRING>
          The API key associated with a Seaplane account used to access Seaplane API endpoints
          
          The value provided here will override any provided in any configuration files.
          A CLI provided value also overrides any environment variables.
          One can use a special value of '-' to signal the value should be read from STDIN.
          
          [env: SEAPLANE_API_KEY]

  -G, --gateway-flight <NAME>
          The name of the Flight to be used as the public traffic gateway that will receive all traffic that arrives on the public URL (if only a single Flight is included in this Formation, it will be implied as the gateway)

      --force
          Override any existing Formation with the same NAME

  -S, --stateless
          Ignore local state files, do not read from or write to them

  -h, --help
          Print help (see a summary with '-h')

  -V, --version
          Print version

FLIGHT SPEC

    The Flight may be specified in one of the following ways

    FLIGHT_SPEC := INLINE-SPEC | PATH | -
    PATH        := PATH is an existing file with a Flight in JSON format
    -           := STDIN will be read for a Flight in JSON format
    INLINE-SPEC := Comma separated LIST of ATTRIBUTE
    ATTRIBUTE   := image=IMAGE [ | name=NAME | minimum=NUM | maximum=NUM | api-permission | architecture=ARCH ]
    NUM         := Positive integer (minimum default is 1 if omitted; maximum default is 'autoscale as needed')
    ARCH        := amd64 | arm64

    NOTE that when using - only one Flight may be provided via STDIN

```
