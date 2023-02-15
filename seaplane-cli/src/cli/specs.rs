pub const FLIGHT_SPEC: &str = "FLIGHT SPEC

    The Flight may be specified in one of the following ways

    FLIGHT_SPEC := INLINE-SPEC | PATH | -
    PATH        := PATH is an existing file with a Flight in JSON format
    -           := STDIN will be read for a Flight in JSON format
    INLINE-SPEC := Comma separated LIST of ATTRIBUTE
    ATTRIBUTE   := image=IMAGE [ | name=NAME | minimum=NUM | maximum=NUM | api-permission | architecture=ARCH ]
    NUM         := Positive integer (minimum default is 1 if omitted; maximum default is 'autoscale as needed')
    ARCH        := amd64 | arm64

    NOTE that when using - only one Flight may be provided via STDIN";

pub const REGION_SPEC: &str = "REGION SPEC

    The regions are based on ISO 3166 alpha-2 continent codes with a few additions to capture
    regulatory differences along with some more intuitive or common aliases. The currently
    supported mappings are:

    XA => Asia
    XC => PRC => PeoplesRepublicofChina
    XE => EU  => Europe
    XF => Africa
    XN => NAmerica => NorthAmerica
    XO => Oceania
    XQ => Antarctica
    XS => SAmerica => SouthAmerica
    XU => UK => UnitedKingdom

    This list is subject to change or expand.";

pub const IMAGE_SPEC: &str = r#"IMAGE SPEC

    NOTE that a default registry of `registry.cplane.cloud` is used.

    Valid images can be defined using the grammar

    reference                       := name [ ":" tag ] [ "@" digest ]
    name                            := [domain '/'] path-component ['/' path-component]*
    domain                          := domain-component ['.' domain-component]* [':' port-number]
    domain-component                := /([a-zA-Z0-9]|[a-zA-Z0-9][a-zA-Z0-9-]*[a-zA-Z0-9])/
    port-number                     := /[0-9]+/
    path-component                  := alpha-numeric [separator alpha-numeric]*
    alpha-numeric                   := /[a-z0-9]+/
    separator                       := /[_.]|__|[-]*/

    tag                             := /[\w][\w.-]{0,127}/

    digest                          := digest-algorithm ":" digest-hex
    digest-algorithm                := digest-algorithm-component [ digest-algorithm-separator digest-algorithm-component ]*
    digest-algorithm-separator      := /[+.-_]/
    digest-algorithm-component      := /[A-Za-z][A-Za-z0-9]*/
    digest-hex                      := /[0-9a-fA-F]{32,}/ ; At least 128 bit digest value

    identifier                      := /[a-f0-9]{64}/
    short-identifier                := /[a-f0-9]{6,64}/

    EXAMPLES

    registry.cplane.cloud/busybox@sha256:7cc4b5aefd1d0cadf8d97d4350462ba51c694ebca145b08d7d41b41acc8db5aa
    registry.cplane.cloud/seaplane/busybox:latest"#;
