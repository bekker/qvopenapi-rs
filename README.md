# qvopenapi-rs

## Build & running on macOS
```sh
# Setup rust cross-compilation
rustup target add i686-pc-windows-gnu

# Install mingw
brew install mingw-w64

# Install wine
brew install --cask wine-stable

# Build webserver example
cargo build --example webserver --release --target i686-pc-windows-gnu
```

## Download DLL
- https://download.nhqv.com/download/iflgtrading/openapi.qv.zip
