#!/bin/sh
./compile.sh &&
tmux new-session -d 'cd catch_server && RUST_BACKTRACE=1 cargo run; cat'
tmux split-window -h 'cd catch_client && RUST_BACKTRACE=1 cargo run; cat'
tmux -2 attach-session -d
