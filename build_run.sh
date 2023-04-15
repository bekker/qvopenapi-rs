#!/bin/bash
# Build webserver example
cargo build -p qvopenapi-http --release --target i686-pc-windows-gnu && \
    wine64 target/i686-pc-windows-gnu/release/qvopenapi-http.exe
    