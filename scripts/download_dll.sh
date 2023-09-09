#!/bin/bash

set -e

SCRIPT_DIR=$( cd -- "$( dirname -- "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )
cd "$SCRIPT_DIR/.."

DLL_URL="https://download.nhqv.com/download/iflgtrading/openapi.qv.zip"
DLL_DIR="./dlls"
DLL_ZIP_FILE="${DLL_DIR}/openapi.qv.zip"
TARGET_DIR="./target/i686-pc-windows-gnu/release"

mkdir -p "$DLL_DIR"
mkdir -p "$TARGET_DIR"

if [ ! -f "$DLL_ZIP_FILE" ]
then
    echo "Downloading DLL..."
    curl -Lf "$DLL_URL" -o "$DLL_ZIP_FILE"
fi

echo "Unzipping DLL..."
if [ "$(uname)" == "Darwin" ]; then
    # unzip on mac can't handle Korean encodings
    ditto -V -x -k --sequesterRsrc --rsrc "$DLL_ZIP_FILE" "$DLL_DIR"
else
    unzip "$DLL_ZIP_FILE" -d "$DLL_DIR"
fi

echo "Copying into release dir..."
cp "${DLL_DIR}/bin"/*.dll "$TARGET_DIR/"

echo "Done!"
