#!/bin/sh
cd catch_server &&
cargo build &&
cd ../catch_client &&
cargo build &&
cd ../
