#!/usr/bin/env -S just --justfile

_default:
  just --list

all: clean build

ci-lint: check test

build *args:
  cargo build {{args}}

check:
  cargo fmt --all -- --check
  cargo clippy --all --all-features -- -D warnings

check-forbidden:
  @bin/forbid

clean:
  cargo clean

test *args:
  RUST_BACKTRACE=1 cargo nextest run {{args}}
