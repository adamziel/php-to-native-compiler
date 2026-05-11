#!/usr/bin/env sh
set -eu
cargo test
cargo run -p phpc -- test
cargo run -p phpc -- test --compare-php
