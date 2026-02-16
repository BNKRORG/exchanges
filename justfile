#!/usr/bin/env just --justfile

fmt:
    cargo +nightly fmt --all -- --config format_code_in_doc_comments=true

check:
    cargo check --all

clippy:
    cargo clippy --all

test:
    cargo test --all

precommit: fmt check clippy test
