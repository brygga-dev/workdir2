#!/bin/bash
cargo build --release
cp ../target/release/mysql-proxy ../../server/base/db-proxy/db-proxy
wop rebuild db-proxy