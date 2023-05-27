#!/bin/bash

if [ -f ".env.local" ]; then
    source .env.local
fi

cargo run -p qvopenapi-http --release --target i686-pc-windows-msvc