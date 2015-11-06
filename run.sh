#!/bin/sh
./compile.sh -j4 --release &&
tmux new-session -d 'bash -c "RUST_LOG=catch_server=debug,catch_shared=debug RUST_BACKTRACE=1 catch_server/target/release/catch_server 2>&1 | tee server.log; bash";'
tmux new-window 'bash -c "RUST_LOG=catch_client=debug,catch_shared=debug RUST_BACKTRACE=1 catch_client/target/release/catch_client 2>&1 | tee client.log; bash";'
tmux -2 attach-session -d
