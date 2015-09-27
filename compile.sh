#!/bin/sh
cargo build "$@" --manifest-path=catch_server/Cargo.toml &&
cargo build "$@" --manifest-path=catch_client/Cargo.toml
