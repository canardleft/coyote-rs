#!/bin/env -S just --justfile

CARGO_ARGS := "--locked"

# verify all of the code
check-and-test: test check

# run test suite
test:
    RUST_LOG=debug cargo nextest run {{ CARGO_ARGS }} --all-features --all-targets
    RUST_LOG=debug cargo test {{ CARGO_ARGS }} --doc --all-features
alias t := test

# generate library documentation
doc: 
    cargo doc {{ CARGO_ARGS }} --document-private-items
alias d := doc

# run static checks
check: check-code check-doc check-deps check-meta check-fmt
alias c := check

# check code for errors/warnings
check-code:
    cargo clippy {{ CARGO_ARGS }}

# check formatting
check-fmt:
    cargo fmt --check
    taplo check .

# check dependencies
check-deps:
    cargo machete
    cargo deny {{ CARGO_ARGS }} check advisories

# check metadata
check-meta:
    lychee .
    codespell .

# check documentation
check-doc:
    cargo doc {{ CARGO_ARGS }} --no-deps

# run an example
run-example example="connecting":
    RUST_LOG=debug cargo run {{ CARGO_ARGS }} --example={{example}}
