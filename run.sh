#!/bin/sh
./compile.sh &&
tmux new-session -d 'cd catch_server && bash -c "RUST_LOG=catch_server=debug,catch_shared=debug RUST_BACKTRACE=1 cargo run; bash"; '
tmux new-window 'cd catch_client && bash -c "RUST_LOG=catch_client=debug,catch_shared=debug RUST_BACKTRACE=1 cargo run; bash"; '
tmux -2 attach-session -d
