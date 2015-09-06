#!/bin/sh
./compile.sh &&
tmux new-session -d 'cd catch_server && bash -c "RUST_BACKTRACE=1 cargo run; bash"; '
tmux split-window -h 'cd catch_client && bash -c "RUST_BACKTRACE=1 cargo run; bash"; '
tmux -2 attach-session -d
