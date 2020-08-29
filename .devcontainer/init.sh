#!/bin/bash

set -e
set -x

rustup component add clippy
rustup component add rustfmt
cargo install cargo-expand