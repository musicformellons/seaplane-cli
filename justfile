set windows-shell := ["powershell.exe", "-NoLogo", "-Command"]

SELF := justfile_directory()
DIST := SELF / 'dist'
BIN_EXE := if os() == 'windows' { '.exe' } else { '' }
CLI_DIR := 'seaplane-cli'
CLI_MANIFEST := CLI_DIR / 'Cargo.toml'
SDK_RUST_DIR := 'seaplane-sdk/rust'
SDK_RUST_MANIFEST := SDK_RUST_DIR / 'Cargo.toml'
IMAGE_REF_MANIFEST := 'crates/container-image-ref/Cargo.toml'
OID_MANIFEST := 'crates/oid/Cargo.toml'
SDK_PYTHON_DIR := 'seaplane-sdk/python'

SHORTSHA := `git rev-parse --short HEAD`
CURRENT_BRANCH := `git rev-parse --abbrev-ref HEAD`

export TARGET := arch()
GON_CONFIG := SELF / 'dist/sign_' + TARGET / 'config.hcl'

# Set the test runner to 'cargo nextest' if not in CI or on Windows
# (Windows ARM64 has an install issue with cargo-nextest at the moment)
TEST_RUNNER := if env_var_or_default("CI", '0') == "1" { 'cargo test' } else { if os() == 'windows' { 'cargo test' } else { 'cargo nextest run' } }
ARG_SEP := if TEST_RUNNER == "cargo nextest run" { '' } else { '--' }

@_default:
    just --list

@about:
    echo "OS: {{ os() }}"
    echo "Family: {{ os_family() }}"
    echo "Arch: {{ arch() }}"
    echo "Rust: $(rustc --version)"
    echo "Cargo: $(cargo --version)"
    echo "Invocation Dir: {{ invocation_directory() }}"

_setup-internal: (_cargo-install 'httpmock --features standalone') (_cargo-install 'cargo-lichking' 'cargo-audit' 'typos-cli')

# Install all needed components and tools
[linux]
setup: _setup-internal (_cargo-install 'cargo-nextest')

# Install all needed components and tools
[windows]
setup: _setup-internal

# Install all needed components and tools
[macos]
setup: _setup-internal (_cargo-install 'cargo-nextest') _install-gon

# Run cargo-audit to scan for vulnerable crates
audit: (_cargo-install 'cargo-audit')
    cargo audit

# Run the CI suite for all SDKs (only runs for your native os/arch!)
ci-sdk: lint-sdk-rust lint-sdk-python lint-sdk-javascript test-sdk-rust test-sdk-python test-sdk-javascript doc

# Run the CI suite for the Rust SDK (only runs for your native os/arch!)
ci-sdk-rust: lint-sdk-rust test-sdk-rust _doc-rust-crate

# Run the CI suite for the Python SDK (only runs for your native os/arch!)
ci-sdk-python: lint-sdk-python test-sdk-python doc-python

# Run the CI suite for the JavaScript SDK (only runs for your native os/arch!)
ci-sdk-javascript: lint-sdk-javascript test-sdk-javascript doc-javascript
    cd seaplane-sdk/javascript; npm ci

# Run the CI suite for the CLI (only runs for your native os/arch!)
ci-cli: lint-cli test-cli

# Run the full CI suite (only runs for your native os/arch!)
ci: audit ci-cli ci-sdk ci-libs-container-image-ref ci-libs-oid

# Run the CI suite for the container-image-ref library
ci-libs-container-image-ref: lint-libs-container-image-ref test-libs-oid (_doc-rust-crate IMAGE_REF_MANIFEST)

# Run the CI suite for the OID library
ci-libs-oid: lint-libs-oid test-libs-oid (_doc-rust-crate OID_MANIFEST)

# Build all documentation
doc: doc-rust doc-python doc-javascript

# Build All Rust documentation
doc-rust: doc-cli _doc-rust-crate (_doc-rust-crate IMAGE_REF_MANIFEST) (_doc-rust-crate OID_MANIFEST)

# Build Rust documentation for the CLI
doc-cli: (_doc-rust-crate CLI_MANIFEST)

# Build Python documentation
doc-python:
    @echo "doc-python: NOT YET IMPLEMENTED"

# Build JavaScript documentation
doc-javascript:
    @echo "doc-javascript: NOT YET IMPLEMENTED"

# Check if code formatter would make changes
fmt-check: fmt-check-cli fmt-check-sdk-rust fmt-check-sdk-python fmt-check-sdk-javascript

# Check if code formatter would make changes to the CLI
fmt-check-cli:
    cargo fmt --manifest-path {{ CLI_DIR / 'Cargo.toml' }} --check

# Check if code formatter would make changes to the Rust SDK
fmt-check-sdk-rust:
    cargo fmt --manifest-path {{ SDK_RUST_DIR / 'Cargo.toml' }} --check

# Check if code formatter would make changes to the container-image-ref library
fmt-check-libs-container-image-ref:
    cargo fmt --manifest-path {{ IMAGE_REF_MANIFEST }} --check

# Check if code formatter would make changes to the OID library
fmt-check-libs-oid:
    cargo fmt --manifest-path {{ OID_MANIFEST }} --check

# Check if code formatter would make changes to the Python SDK
fmt-check-sdk-python: _python-setup
    cd seaplane-sdk/python/; poetry run nox -s fmt_check

# Check if code formatter would make changes to the JavaScript SDK
fmt-check-sdk-javascript:
    @echo "fmt-check-sdk-javascript: NOT YET IMPLEMENTED"

# Format all the code
fmt: fmt-sdk-rust fmt-cli fmt-sdk-python fmt-sdk-javascript

# Format the CLI code
fmt-cli:
    cargo fmt --manifest-path {{ CLI_MANIFEST }}

# Format the Rust SDK code
fmt-sdk-rust:
    cargo fmt --manifest-path {{ SDK_RUST_MANIFEST }}

# Format the library container-image-ref code
fmt-libs-container-image-ref:
    cargo fmt --manifest-path {{ IMAGE_REF_MANIFEST }}

# Format the library OID code
fmt-libs-oid:
    cargo fmt --manifest-path {{ OID_MANIFEST }}

# Format the Python SDK code
fmt-sdk-python: _python-setup
    cd seaplane-sdk/python/; poetry run nox -s fmt

# Format the JavaScript SDK code
fmt-sdk-javascript:
    @echo "fmt-sdk-javascript: NOT YET IMPLEMENTED"

# Run all checks and lints
lint: lint-sdk-rust lint-sdk-python lint-sdk-javascript lint-cli lint-libs-oid lint-libs-container-image-ref

# Run all lint checks against the CLI
lint-cli: spell-check fmt-check-cli (_lint-rust-crate CLI_MANIFEST '--no-default-features')

# Run all lint checks against the Rust SDK
lint-sdk-rust: spell-check fmt-check-sdk-rust _lint-rust-crate (_lint-rust-crate SDK_RUST_MANIFEST '--features unstable')

# Run all lint checks against the Python SDK
lint-sdk-python: spell-check fmt-check-sdk-python
    cd seaplane-sdk/python/; poetry run nox -s lint
    cd seaplane-sdk/python/; poetry run nox -s type_check

# Run all lint checks against the JavaScript SDK
lint-sdk-javascript: spell-check fmt-check-sdk-javascript
    cd seaplane-sdk/javascript; npm run lint

# Run all lint checks against the library container-image-ref
lint-libs-container-image-ref: fmt-check-libs-container-image-ref (_lint-rust-crate IMAGE_REF_MANIFEST)

# Run all lint checks against the library OID
lint-libs-oid: fmt-check-libs-oid (_lint-rust-crate OID_MANIFEST)

# Run basic integration and unit tests for all Rust crates
test-rust: test-sdk-rust (_test-rust-crate CLI_MANIFEST) (_test-rust-api-crate CLI_MANIFEST)

# Run basic integration and unit tests for the CLI
test-cli: (_doc-rust-crate CLI_MANIFEST) (_test-rust-crate CLI_MANIFEST) (_test-rust-api-crate CLI_MANIFEST) test-ui

# Run basic integration and unit tests for the Rust SDK
test-sdk-rust: _test-rust-crate _test-rust-api-crate (_test-rust-api-crate SDK_RUST_MANIFEST ',compute_api_v2,unstable') _test-rust-doc-crate _doc-rust-crate

# Run basic integration and unit tests for the library container-image-ref
test-libs-container-image-ref: (_test-rust-crate IMAGE_REF_MANIFEST) (_test-rust-doc-crate IMAGE_REF_MANIFEST)

# Run basic integration and unit tests for the OID library
test-libs-oid: (_test-rust-crate OID_MANIFEST '' '-D warnings') (_test-rust-doc-crate OID_MANIFEST)

# Run basic integration and unit tests for the Python SDK
test-sdk-python: _python-setup
    cd seaplane-sdk/python/; poetry run nox -s test

# Run basic integration and unit tests for the JavaScript SDK
test-sdk-javascript:
    cd seaplane-sdk/javascript/; npm test

# Run UI tests
test-ui $RUSTFLAGS='-D warnings':
    {{ TEST_RUNNER }}  --features ui_tests --manifest-path {{ CLI_MANIFEST }}
    {{ TEST_RUNNER }}  --features unstable,semantic_ui_tests --manifest-path {{ CLI_MANIFEST }}

# Update all third party licenses
update-licenses: (_cargo-install 'cargo-lichking')
    cargo lichking bundle --variant name-only > {{ CLI_DIR / 'share/third_party_licenses.md' }}

# Spell check the entire repo
spell-check: _install-spell-check
    typos

#
# Python Helpers
#
_python-setup:
    cd seaplane-sdk/python/; poetry install

#
# Rust Helpers
#

# Run basic integration and unit tests for a Rust crate
_test-rust-crate MANIFEST=SDK_RUST_MANIFEST FEATURES='' $RUSTFLAGS='-D warnings':
    {{ TEST_RUNNER }} --no-default-features --manifest-path {{ MANIFEST }}
    {{ TEST_RUNNER }} {{ FEATURES }} --manifest-path {{ MANIFEST }}

# build documentation for a Rust crate
_doc-rust-crate MANIFEST=SDK_RUST_MANIFEST $RUSTDOCFLAGS="-D warnings":
    cargo doc --manifest-path {{ MANIFEST }} --no-deps --all-features --document-private-items

# Lint a Rust crate
_lint-rust-crate MANIFEST=SDK_RUST_MANIFEST RUST_FEATURES='':
    cargo clippy --no-default-features --manifest-path {{ MANIFEST }} --all-targets -- -D warnings
    cargo clippy --all-features --manifest-path {{ MANIFEST }} --all-targets -- -D warnings
    cargo clippy {{ RUST_FEATURES }} --manifest-path {{ MANIFEST }} --all-targets -- -D warnings

# Run API tests using a mock HTTP server
_test-rust-api-crate MANIFEST=SDK_RUST_MANIFEST EXTRA_FEATURES='' $RUSTFLAGS='-D warnings':
    {{ TEST_RUNNER }} --features api_tests{{ EXTRA_FEATURES }} --manifest-path {{ MANIFEST }} {{ ARG_SEP }} --test-threads=1
    {{ TEST_RUNNER }} --features unstable,api_tests{{ EXTRA_FEATURES }} --manifest-path {{ MANIFEST }} {{ ARG_SEP }} --test-threads=1

# Run documentation tests
_test-rust-doc-crate MANIFEST=SDK_RUST_MANIFEST FEATURES='':
    cargo test --doc --manifest-path {{ MANIFEST }} {{ FEATURES }}
    cargo test --doc --manifest-path {{ MANIFEST }} --no-default-features
    cargo test --doc --manifest-path {{ MANIFEST }} --all-features

#
# Small Helpers
#

[unix]
_install-spell-check:
    #!/usr/bin/env bash
    set -euo pipefail
    if ! command -v typos 2>&1 >/dev/null ; then
      cargo install typos-cli --force
    fi

[windows]
_install-spell-check:
    #!powershell.exe
    $ret = Get-Command typos >$null 2>$null

    if (! $?) {
      cargo install typos-cli --force
    }

# List TODO items in current branch only
todos-in-branch:
    git diff --name-only {{ CURRENT_BRANCH}} $(git merge-base {{ CURRENT_BRANCH }} main) | xargs rg -o 'TODO:.*$'

# List 'TODO:' items
todos:
    rg -o 'TODO:.*$' -g '!justfile'

# Create a nightly CLI release package (latest commit)
package-nightly:
    just _package-build "cli-{{ SHORTSHA }}"

# Create a CLI release package (latest 'cli-v*' tag)
[unix]
package-release:
    just _package-build "$(git tag --list | grep cli-v | sort | tail -n1)"

# Create a CLI release package (latest 'cli-v*' tag)
[windows]
package-release:
    just _package-build "$(git tag --list | findstr cli-v | sort | select -last 1)"

#
# Private/Internal Items
#

# Get the latest short SHA commit for the just CLI directory
_git-shortsha-cli:
    @git --no-pager log -n1 --pretty=format:%h $(dirname {{ CLI_DIR }})

[windows]
_package-build TAG=SHORTSHA:
    #!powershell.exe
    $BUILDDIR = "target/release/".trim()
    $DISTDIR = "dist/".trim()
    cargo build --release --manifest-path {{ CLI_MANIFEST }}
    Remove-Item -Force -Recurse "$DISTDIR/" 2>$null
    New-Item -ItemType Directory -Force -Path "$DISTDIR/bin/" >$null
    New-Item -ItemType Directory -Force -Path "$DISTDIR/share/" >$null
    New-Item -ItemType Directory -Force -Path "$DISTDIR/share/doc/" >$null
    New-Item -ItemType Directory -Force -Path "$DISTDIR/share/doc/seaplane/" >$null
    Remove-Item -Force "$DISTDIR/bin/*"
    Copy-Item "$BUILDDIR/seaplane{{ BIN_EXE }}" "$DISTDIR/bin/"
    Copy-Item seaplane-cli/share/third_party_licenses.md "$DISTDIR/share/doc/seaplane/"
    Copy-Item LICENSE "$DISTDIR/share/doc/seaplane/"
    cd "$DISTDIR"
    $compress = @{
        Path = ".\bin\", ".\share\"
        DestinationPath = "..\seaplane-{{ TAG }}-$("$($Env:PROCESSOR_ARCHITECTURE)".ToLower())-windows.zip"
    }
    Compress-Archive @compress

[unix]
_package-build TAG=SHORTSHA:
    #!/usr/bin/env bash
    set -euo pipefail
    BUILDDIR=target/release/
    DISTDIR=dist/
    cargo build --release --manifest-path {{ CLI_MANIFEST }}
    mkdir -p ${DISTDIR}/{bin,share/doc/seaplane/}
    rm -rf ${DISTDIR}/bin/*
    cp ${BUILDDIR}/seaplane{{ BIN_EXE }} ${DISTDIR}/bin/
    cp seaplane-cli/share/third_party_licenses.md ${DISTDIR}/share/doc/seaplane/
    cp LICENSE ${DISTDIR}/share/doc/seaplane/
    cd ${DISTDIR}
    tar czf ../seaplane-{{ TAG }}-$(uname -m)-{{ os() }}.tar.gz ./*

_cargo-install +TOOLS:
    cargo install {{ TOOLS }}

[macos]
_install-gon:
    #!/usr/bin/env bash
    if ! command -v gon 2>&1 >/dev/null ; then brew install mitchellh/gon/gon; fi

# Sign and notarize a release for macOS
[macos]
_sign $AC_PASSWORD TAG=SHORTSHA SIGNER='${USER}': _install-gon
    #!/usr/bin/env bash
    set -euo pipefail
    DISTDIR={{ justfile_directory() }}/dist
    SIGNDIR=${DISTDIR}/sign_${TARGET}/
    CARGOTGTDIR={{ justfile_directory() }}/target
    ARTIFACTSDIR=${CARGOTGTDIR}/${TARGET}-apple-darwin/release/
    echo Cleaning previous runs
    rm -rf $SIGNDIR
    cargo clean
    mkdir -p $SIGNDIR
    echo Generating Config...
    echo 'source = ["./seaplane"]' >> {{GON_CONFIG}}
    echo 'bundle_id = "io.Seaplane.seaplane"' >> {{GON_CONFIG}}
    echo 'apple_id {' >> {{GON_CONFIG}}
    echo "  username = \"${USER}@seaplane.io\"" >> {{GON_CONFIG}}
    echo "  password = \"$AC_PASSWORD\"" >> {{GON_CONFIG}}
    echo '}' >> {{GON_CONFIG}}
    echo 'sign {' >> {{GON_CONFIG}}
    echo '  application_identity = "663170B344CE42EF1F583807B756239878A92FC8"' >> {{GON_CONFIG}}
    echo '}' >> {{GON_CONFIG}}
    echo 'zip {' >> {{GON_CONFIG}}
    echo "  output_path = \"seaplane-cli-{{ TAG }}-${TARGET}-macos.zip\"" >> {{GON_CONFIG}}
    echo '}' >> {{GON_CONFIG}}
    echo Compiling ${TARGET}...
    cargo --quiet build --release --manifest-path seaplane-cli/Cargo.toml --target ${TARGET}-apple-darwin
    echo Copying binaries...
    cp ${ARTIFACTSDIR}/seaplane ${SIGNDIR}
    echo Signing...
    cd ${SIGNDIR}; gon config.hcl
    echo Done!
    echo Saving Artifacts to ${DISTDIR}
    cp ${SIGNDIR}/seaplane-cli-{{ TAG }}-${TARGET}-macos.zip ${DISTDIR}

