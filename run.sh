#!/bin/sh
./compile.sh "$@" &&
tmux new-session -d 'bash -c "RUST_LOG=catch_server=info,catch_shared=debug RUST_BACKTRACE=1 cargo run $@ --manifest-path=catch_server/Cargo.toml; bash";'
tmux new-window 'bash -c "RUST_LOG=catch_client=info,catch_shared=debug RUST_BACKTRACE=1 cargo run $@ --manifest-path=catch_client/Cargo.toml; bash";'
tmux -2 attach-session -d
