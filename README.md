# qvopenapi-rs
- (아래는 목표)
- NH투자증권에서 제공하는 QV Open API를 Rust에서 사용할 수 있도록 wrapping
- 편하게 사용할 수 있도록 자동으로 Windows 애플리케이션을 실행 및 이벤트를 관리하는 라이브러리
- REST API 형태로 호출할 수 있는 웹서버 애플리케이션
- Wine을 통해 리눅스에서 실행할 수 있는 Docker 이미지

## Current Status
- 아직 아무것도 안됨!
- [ ] DLL wrapping
- [ ] 편하게 사용할 수 있도록 자동으로 Windows 애플리케이션을 실행 및 이벤트를 관리하는 라이브러리
- [ ] REST API 형태로 호출할 수 있는 웹서버 애플리케이션
- [ ] Wine을 통해 리눅스에서 실행할 수 있는 Docker 이미지

## Modules
- qvopenapi-sys: `wmca.dll`을 wrapping해둔 라이브러리
- qvopenapi: `wmca.dll`은 윈도우 이벤트 기반으로 동작하므로, 편하게 쓸 수 있도록 Windows 애플리케이션을 자동으로 관리하고 주요 Tx 들을 별도 메소드로 제공하는 라이브러리
- qvopenapi-http: 다른 언어로 작성된 애플리케이션과 통신하기 편하도록 REST API 형태의 인터페이스 제공

## Build & running on macOS
```sh
# Setup rust cross-compilation
rustup target add i686-pc-windows-gnu

# Install mingw
brew install mingw-w64

# Install wine crossover
brew tap gcenx/wine
brew install --cask --no-quarantine wine-crossover

# Build webserver
# https://github.com/rust-lang/rust/issues/79609
# To bypass missing 'dwarf' exception handling on mingw i686 installations,
# set panic = abort and provide mock unwind symbol to the linker
cargo rustc -p qvopenapi-http --release --target i686-pc-windows-gnu --features "disable-unwind" -- -C "panic=abort"

# Run using wine
wine target/i686-pc-windows-gnu/release/qvopenapi-http.exe
```

## Build & running on Windows
```sh
# Setup rust cross-compilation
rustup target add i686-pc-windows-msvc

# Build webserver
cargo build -p qvopenapi-http --release --target i686-pc-windows-msvc

# Run webserver
cargo run -p qvopenapi-http --release --target i686-pc-windows-msvc
```

## Download DLL
- https://download.nhqv.com/download/iflgtrading/openapi.qv.zip
