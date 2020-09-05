#!/bin/bash

set -e
set -x

rustup component add clippy
rustup component add rustfmt
cargo install cargo-expand

# Cargo expand requires nightly toolchain
rustup toolchain install nightly