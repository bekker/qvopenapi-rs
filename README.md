# qvopenapi-rs

## Build & running on macOS
```sh
# Setup rust cross-compilation
rustup target add i686-pc-windows-gnu

# Install mingw
brew install mingw-w64

# Install wine staging
brew install --cask --no-quarantine homebrew/cask-versions/wine-staging

# Build webserver example
cargo build --example webserver --release --target i686-pc-windows-gnu

# Run using wine64
wine64 target/i686-pc-windows-gnu/release/examples/webserver.exe
```

## Recommended wine version
- 8.4

## Download DLL
- https://download.nhqv.com/download/iflgtrading/openapi.qv.zip

