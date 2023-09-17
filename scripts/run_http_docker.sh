#!/bin/bash
SCRIPT_DIR=$( cd -- "$( dirname -- "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )
cd "$SCRIPT_DIR/.."

set -e

cargo rustc -p qvopenapi-http --release --target i686-pc-windows-gnu --features "disable-unwind" -- -C "panic=abort"
docker build . -t qvopenapi-http --platform linux/amd64

docker rm qvopenapi-http || docker run -p 18000:18000 --rm -it --name qvopenapi-http qvopenapi-http
