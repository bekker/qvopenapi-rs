#!/bin/bash

if [ -f ".env.local" ]; then
    source .env.local
fi

cargo rustc -p qvopenapi-future --example "$1" --release --target i686-pc-windows-gnu --features "disable-unwind" -- -C "panic=abort" && \
    cp target/i686-pc-windows-gnu/release/examples/connect_future.exe target/i686-pc-windows-gnu/release/example_connect_future.exe && \
    wine target/i686-pc-windows-gnu/release/example_connect_future.exe
