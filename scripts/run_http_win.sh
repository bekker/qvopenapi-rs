#!/bin/bash
SCRIPT_DIR=$( cd -- "$( dirname -- "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )
cd "$SCRIPT_DIR/.."

if [ -f ".env.local" ]; then
    source .env.local
fi

if [ ! -f "./target/i686-pc-windows-gnu/release/wmca.dll" ]
then
    $SCRIPT_DIR/download_dll.sh
fi

cargo run -p qvopenapi-http --release --target i686-pc-windows-msvc
