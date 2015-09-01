#!/bin/sh
tmux new-session -d 'cd catch_server && cargo run; cat'
tmux split-window -h 'cd catch_client && cargo run; cat'
tmux -2 attach-session -d
