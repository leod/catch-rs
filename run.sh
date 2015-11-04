#!/bin/sh
./compile.sh -j4 &&
tmux new-session -d 'bash -c "RUST_LOG=catch_server=info,catch_shared=info RUST_BACKTRACE=1 catch_server/target/debug/catch_server 2>&1 | tee server.log; bash";'
tmux new-window 'bash -c "RUST_LOG=catch_client=info,catch_shared=info RUST_BACKTRACE=1 catch_client/target/debug/catch_client 2>&1 | tee client.log; bash";'
tmux -2 attach-session -d
