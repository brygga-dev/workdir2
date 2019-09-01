#!/bin/bash
cargo build --release
cp ../target/release/wp-cli-server ../../server/wp-cli/wp-cli-server
