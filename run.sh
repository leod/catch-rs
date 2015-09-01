#!/bin/sh
tmux new-session -d 'cd catch_server && cargo run; sh'
tmux split-window -h 'cd catch_client && cargo run; sh'
tmux -2 attach-session -d
