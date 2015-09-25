#!/bin/sh
cd catch_server &&
cargo build --release &&
cd ../catch_client &&
cargo build --release &&
cd ../
