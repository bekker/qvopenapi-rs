#!/bin/bash
SCRIPT_DIR=$( cd -- "$( dirname -- "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )
cd "$SCRIPT_DIR/.."

set -e

CERT_DIR_HOST=`realpath ~/.wine/drive_c/users/crossover/AppData/LocalLow/NPKI`
CERT_DIR_CONTAINER="/namu/NPKI"

cargo rustc -p qvopenapi-http --release --target i686-pc-windows-gnu --features "disable-unwind" -- -C "panic=abort"
docker build . -t qvopenapi-http --platform linux/amd64

docker rm qvopenapi-http || docker run -p 18000:18000 -v "${CERT_DIR_HOST}:${CERT_DIR_CONTAINER}" --rm -it --name qvopenapi-http qvopenapi-http
