# Instructions for AI Agents

## What is This Codebase?

This tool, `bellhop`, makes importing Debian packages into a set of `aptly`-managed repositories
easier.

## Build System

All the standard Cargo commands apply but with one important detail: make sure to add `--all-features` so that
both the async and blocking client are built, tested, linted, and so on.

 * `cargo build --all-features` to build
 * `cargo nextest run --all-features` to run tests
 * `cargo clippy --all-features` to lint
 * `cargo fmt` to reformat

Always run `cargo check --all-features` before making changes to verify the codebase compiles cleanly.
If compilation fails, investigate and fix compilation errors before proceeding with any modifications.

## Debian Distributions Targeted

See `bellhop::deb::DistributionAlias#all()` for a list of targeted distributions.

On the command line, distributions are specified using name aliases such as `bullseye`, `trixie`, `noble`, `jammy`, etc.

## Key Files

 * `src/main.rs`: the entry point
 * `src/aptly.rs`: functions that drive `aptly` commands
 * `src/cli.rs`: `clap`-based command-line interface definition
 * `src/handlers.rs`: CLI commands are dispatched to these functions
 * Error types: `src/errors.rs` 
 * Shared types: `src/common.rs`
 * `src/deb.rs`: Debian-specific types (e.g. Debian and Ubuntu distributions, distribution aliases)

## Test Suite Layout

 * `tests/*.rs`: integration test modules
 * `tests/test_helpers.rs` contains helper functions shared by multiple test modules

Use `cargo nextest run --profile default --all-features '--' --exact [test module name]` to run
all tests in a specific module.


## Source of Domain Knowledge

 * [`aptly` documentation](https://www.aptly.info/doc/overview/)
 * [`aptly` source code](https://github.com/aptly-dev/aptly)
 * [RabbitMQ Debian and Ubuntu Installation Guide](https://www.rabbitmq.com/docs/install-debian)

Treat this documentation as the ultimate first party source of truth.


## Code Style

Only add very important comments to the tests and the implementation.
All tests must go into new modules under `tests/*.rs`, never into the implementation module.

Use `use` statements at the top module level, never in individual function's scope.
Avoid fully qualified type paths such as `std::fmt::Display` unless they help avoid ambigiuity.


## Git Commits

 * Do not commit changes automatically without an explicit permission to do so
 * Never add yourself as a git commit coauthor
