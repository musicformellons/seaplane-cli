# Contributing to the Seaplane CLI and SDKs

Contributions are welcome!

## Requirements

All projects in this repository require [`just`][just] (v1.8.0 or greater)
 as a command runner.

Additionally, see the sub-project README for details on the specific project requirements.

You can install `just` if you have a Rust toolchain installed via:

```console
cargo install just
```

Otherwise, see the `just` repository for pre-built binaries or how to install for your specific OS.

Once `just` is installed, you can install the other tooling we use to build
and test the system in one command with:

```console
just setup
```

### macOS

macOS additionally requires `brew` to be installed if creating and signing releases.

### Windows

> **Note**
> When running some recipes like `spell-check`, if you receive an error about Execution Policy you
> may need to run the following command. Some `just` recipes run as generated PowerShell scripts which
> require the `unrestricted` policy.
>
> ```powershell
> Set-ExecutionPolicy -ExecutionPolicy unrestricted -Scope CurrentUser
> ```
>
> See https:/go.microsoft.com/fwlink/?LinkID=135170 for more details.

## Workflow

Our workflow is:

- checkout the latest version of the `main` branch
- Create a new branch for your work
- Make your changes
- Create a Pull Request (aka a PR) against `main` as the base branch
- Enjoy the new feature/fix!

There are a couple points we should zoom in on in order to make your contributions as smooth as possible.

- When to open the PR
- What makes a great PR
- Local Tests
- Our Merge Strategy
- What to expect during the PR

### When to Open a Pull Request?

You can open a pull request right away! However, if you're unsure if your patch will be accepted,
or would like to discuss design approaches prior to spending time coding we commend opening an
Issue or Github Discussion where we can assist.

### What Makes a Great PR?

We love all contributions! However, there are a few things you can do to really make your PR top
notch and easy to merge.

- Use the PR Template to give us as much detail as you can about
  - Why you're making the change you're making
  - What components you added or changed (This makes it easier for us to assign the appropriate
    people, or helps us to know what areas to keep watch on in order to review the work and help
    you along the process)
- Ensure all local tests pass (we'll discuss this in the next section)
- Write any appropriate unit or integration tests to go along with your work

### Local Tests

Our Github Continuous Integration (CI) testing will ensure everything is working as expected, but to
save time waiting for the CI servers to process the PR it can help to ensure all the tests work on
your local machine first.

Keep in mind that our tool ships on several operating systems, and system architectures so if you
have the ability to test in those environments as well that's wonderful! However, if you don't
that's no problem; ensuring the tests pass on your local machine is a great starting place where we
can let the CI servers test across the matrix of Operating Systems and Architectures.

The full support matrix can be found in the [docs/ARCHITECTURE.md][architecture] file of this
repository.

There are two ways to run our tests, either using `just` (recommended) or manually.

#### Local Tests Using [`just`](https://github.com/casey/just)

If you have `just` you can run a test suite that mimics much of our CI, but locally and only for
your native architecture.

To run the full test suite use:

```console
just ci
```

If your PR only affects the CLI, or a specific SDK there are recipes to run the test suites for only those
components.

```console
$ just ci-cli
  .. run CLI test suite

$ just ci-sdk-rust
  .. run Rust SDK test suite
```

All of these run the full gamut of tests, even doc tests, `clippy`, and `rustfmt`.
Other recipes exist for smaller or more targeted operations.

To see the full list of recipes use `just` by itself:

```console
$ just
Available recipes:
    about
    audit                         # Run cargo-audit to scan for vulnerable crates
    ci                            # Run the full CI suite (only runs for your native os/arch!)
...
```

If you've made changes to any of the `Cargo.toml` files it's probably a good idea to also update
the third party licenses by:

```console
just update-licenses
```

### Our Merge Strategy

We use the [Squash and Merge][squash] strategy. This means that all the commits you make prior to,
or after opening a PR will be squashed down into a single all encompassing commit. This commit will
then be a "merge commit" on the `main` branch.

This has a few implications for your workflow.

First, prior to opening the PR it's a good idea to make sure you're working off the latest `main`
branch. This can be done with a `rebase` command. For example,

```console
$ git switch my-pr-branch
[..]

$ git rebase main
[..]
```

If you've already opened the PR, and the reviewer asks you to "Rebase onto the latest `main`." This
is what they mean. The only difference, is since you've already push `my-pr-branch` once, after
doing the `git rebase main` command, you'll have to `git push --force`.

Second, having a good and up-to-date summary comment at the top of the PR is crucial, as that will
most likely get copied verbatim (along with any commit message headers) as the merge commit message.

### What to Expect During a PR

So you've opened a PR! Yay!

If this is your first contribution, a Seaplane employee or trusted
contributor will need to give approval to the CI servers to start testing your contribution. Once
you've made some contributions, you'll be added to a list that where CI is allowed to run
automatically.

A Seaplane employee or contributor will get assigned to your PR as a reviewer. If there are any
questions, or changes that need to be considered the reviewer will make a comment on the PR and
let you know.

If changes are requested, or design questions are asked this isn't a bad thing! We want to ensure
we understand the code like you do, and may have additional edge cases we'd like to address, or
maybe we just want to say awesome work!

This back and forth between the reviewer and you may go on for a little while depending on the size
and scope of your contribution. It may take only a few minutes, or a few hours, or in large changes
it could last a few days. In part it depends on the combined availability of both your reviewer and
yourself. There is no timeline, and there is no rush! Take your time and enjoy!

Once the reviewer is finished requesting changes or doesn't have any remaining questions, they will
mark the PR as "Approved." This may kick off additional CI checks, or perhaps one final round from
the CI servers to ensure everything is as it should be.

If all checks are green, the code will be "Squashed and Merged."

Congratulations, you're now a Seaplane Contributor! Welcome Aboard!

[//]: # (Links)

[architecture]: ./ARCHITECTURE.md#support-matrix
[squash]: https://docs.gitlab.com/ee/user/project/merge_requests/squash_and_merge.html
[just]: https://github.com/casey/just
[choco]: https://chocolatey.org/install
