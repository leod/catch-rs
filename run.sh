#!/bin/sh
echo Compiling $1
./compile.sh $1 &&
tmux new-session -d 'bash -c "RUST_LOG=catch_server=debug,catch_shared=debug RUST_BACKTRACE=1 cargo run $1 --manifest-path=catch_client/Cargo.toml; bash"; '
tmux new-window 'bash -c "RUST_LOG=catch_client=debug,catch_shared=debug RUST_BACKTRACE=1 cargo run $1 --manifest-path=catch_server/Cargo.toml; bash"; '
tmux -2 attach-session -d
