#!/bin/bash
# Build webserver example
cargo build -p qvopenapi-webserver --release --target i686-pc-windows-gnu && \
    wine64 target/i686-pc-windows-gnu/release/examples/webserver.exe
    