#!/bin/bash
# Build webserver example

if [ -f ".env.local" ]; then
    source .env.local
fi

cargo rustc -p qvopenapi-http --release --target i686-pc-windows-gnu --features "disable-unwind" -- -C "panic=abort" && \
    wine target/i686-pc-windows-gnu/release/qvopenapi-http.exe
    