#!/bin/sh
set -o errexit

touch build.rs
cargo build --no-default-features --features=generate
echo "Generated '<OUT_DIR>/ffi.rs'. You may want to copy this into 'generated/' with an appropriate name."
