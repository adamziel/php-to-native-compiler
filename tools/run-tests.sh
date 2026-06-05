#!/usr/bin/env sh
set -eu
RUST_TEST_THREADS="${RUST_TEST_THREADS:-1}" cargo test
cargo run -p phpc -- test
cargo run -p phpc -- test --compare-php
