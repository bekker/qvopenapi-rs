#!/bin/bash
# Build webserver example
cargo rustc -p qvopenapi-http --release --target i686-pc-windows-gnu --features "disable-unwind" -- -C "panic=abort" && \
    wine target/i686-pc-windows-gnu/release/qvopenapi-http.exe
    