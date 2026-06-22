#!/usr/bin/env bash

set -e
RELEASE=$1

cargo run $RELEASE --bin streamfy -- topic delete echo || true
cargo run $RELEASE --bin streamfy -- topic create echo

cargo run $RELEASE --bin echo
