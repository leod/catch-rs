#!/bin/sh
./compile.sh "$@" --release &&
tmux new-session -d 'bash -c "RUST_LOG=catch_server=info,catch_shared=info RUST_BACKTRACE=1 catch_server/target/release/catch_server 2>&1 | tee server.log; bash";'
tmux new-window 'bash -c "RUST_LOG=catch_client=info,catch_shared=info RUST_BACKTRACE=1 catch_client/target/release/catch_client 2>&1 | tee client.log; bash";'
tmux -2 attach-session -d
