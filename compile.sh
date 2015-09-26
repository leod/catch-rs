#!/bin/sh
cd catch_server &&
cargo build $1 &&
cd ../catch_client &&
cargo build $1 &&
cd ../
