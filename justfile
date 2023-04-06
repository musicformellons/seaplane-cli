SELF := justfile_directory()
DIST := SELF / 'dist'
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

_setup-internal: (_cargo-install 'httpmock --features standalone') (_cargo-install 'cargo-lichking' 'cargo-audit')

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

# Run the full CI suite (only runs for your native os/arch!)
ci: audit ci-sdk ci-libs-container-image-ref ci-libs-oid

# Run the CI suite for the container-image-ref library
ci-libs-container-image-ref: lint-libs-container-image-ref test-libs-oid (_doc-rust-crate IMAGE_REF_MANIFEST)

# Run the CI suite for the OID library
ci-libs-oid: lint-libs-oid test-libs-oid (_doc-rust-crate OID_MANIFEST)

# Build all documentation
doc: doc-rust doc-python doc-javascript

# Build All Rust documentation
doc-rust: _doc-rust-crate (_doc-rust-crate IMAGE_REF_MANIFEST) (_doc-rust-crate OID_MANIFEST)

# Build Python documentation
doc-python:
    @echo "doc-python: NOT YET IMPLEMENTED"

# Build JavaScript documentation
doc-javascript:
    @echo "doc-javascript: NOT YET IMPLEMENTED"

# Check if code formatter would make changes
fmt-check: fmt-check-sdk-rust fmt-check-sdk-python fmt-check-sdk-javascript

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
fmt: fmt-sdk-rust fmt-sdk-python fmt-sdk-javascript

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
lint: lint-sdk-rust lint-sdk-python lint-sdk-javascript lint-libs-oid lint-libs-container-image-ref

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
test-rust: test-sdk-rust

# Run basic integration and unit tests for the Rust SDK
test-sdk-rust: _test-rust-crate _test-rust-api-crate (_test-rust-api-crate SDK_RUST_MANIFEST ',locks_api_v1,compute_api_v2,restrict_api_v1,identity_api_v1,metadata_api_v1') _test-rust-doc-crate _doc-rust-crate

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

# Update all third party licenses
update-licenses: (_cargo-install 'cargo-lichking')
    cargo lichking bundle --variant name-only > third_party_licenses.md

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

#
# Private/Internal Items
#

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

# Generate an OID using the specified prefix and UUID (UUID is randomly generated v4 if omitted)
_gen-oid prefix="frm" uuid='': (_cargo-install 'cargo-script')
    #!/usr/bin/env run-cargo-script
    //! ```cargo
    //! [dependencies]
    //! seaplane-oid = "0.4.0"
    //! uuid = { version = "1.2.2", features=["v4"] }
    //! ```
    fn main() {
        let uuid_str = "{{ uuid }}";
        let id = if uuid_str.is_empty() {
            uuid::Uuid::new_v4()
        } else {
            uuid_str.parse::<uuid::Uuid>().unwrap()
        };
        let oid = seaplane_oid::Oid::with_uuid("{{ prefix }}", id)
            .expect("hey, that might not have been a valid uuid!");

        println!("{oid}");
    }

