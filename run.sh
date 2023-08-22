#!/usr/bin/env bash

RUSTFLAGS="-C target-cpu=native" cargo build --release
./target/release/blazing-bot
