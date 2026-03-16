#!/bin/env -S just --justfile

# verify all of the code
check-and-test: test check

# run test suite
test:
    cargo nextest run --all-features --all-targets
    cargo test --doc --all-features
alias t := test

# generate library documentation
doc: 
    cargo doc --document-private-items
alias d := doc

# run static checks
check: check-code check-doc check-deps check-meta check-fmt
alias c := check

# check code for errors/warnings
check-code:
    cargo check

# check formatting
check-fmt:
    cargo fmt --check
    taplo check .

# check dependencies
check-deps:
    cargo machete
    cargo deny check advisories

# check metadata
check-meta:
    lychee .
    codespell .

# check documentation
check-doc:
    cargo doc --no-deps

# run an example
run-example example="connecting":
    RUST_LOG=debug cargo run --example={{example}}
