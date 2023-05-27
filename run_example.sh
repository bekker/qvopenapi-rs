#!/bin/bash
# Build webserver example
cargo rustc -p qvopenapi --example connect --release --target i686-pc-windows-gnu --features "disable-unwind" -- -C "panic=abort" && \
    cp target/i686-pc-windows-gnu/release/examples/connect.exe target/i686-pc-windows-gnu/release/example_connect.exe && \
    wine target/i686-pc-windows-gnu/release/example_connect.exe
