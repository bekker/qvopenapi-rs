# qvopenapi-rs
- (아래는 목표)
- NH투자증권에서 제공하는 QV Open API를 Rust에서 사용할 수 있도록 wrapping
- 편하게 사용할 수 있도록 자동으로 Windows 애플리케이션을 실행 및 이벤트를 관리하는 라이브러리
- REST API 형태로 호출할 수 있는 웹서버 애플리케이션
- Wine을 통해 리눅스에서 실행할 수 있는 Docker 이미지

## Current Status
- [x] DLL wrapping
- [x] 편하게 사용할 수 있도록 자동으로 Windows 애플리케이션을 실행 및 이벤트를 관리하는 라이브러리
- [ ] REST API 형태로 호출할 수 있는 웹서버 애플리케이션
- [ ] Wine을 통해 리눅스에서 실행할 수 있는 Docker 이미지

## Modules
- qvopenapi-sys: `wmca.dll`을 wrapping해둔 라이브러리
- qvopenapi: `wmca.dll`은 윈도우 이벤트 기반으로 동작하므로, 편하게 쓸 수 있도록 Windows 애플리케이션을 자동으로 관리하고 주요 Tx 들을 별도 메소드로 제공하는 라이브러리
- qvopenapi-http: 다른 언어로 작성된 애플리케이션과 통신하기 편하도록 REST API 형태의 인터페이스 제공

## How to build

### 플랫폼별 cross compile 구성
- DLL이 i686 바이너리에서만 로드되기 때문에, i686 크로스 컴파일 설정 필요

#### macOS
```sh
# Setup rust cross-compilation
rustup target add i686-pc-windows-gnu

# Install build tools
brew install mingw-w64 llvm

# Install wine crossover
brew tap gcenx/wine
brew install --cask --no-quarantine wine-crossover

# Download dlls
scripts/download_dll.sh

# Build webserver
# https://github.com/rust-lang/rust/issues/79609
# To bypass missing 'dwarf' exception handling on mingw i686 installations,
# set panic = abort and provide mock unwind symbol to the linker
cargo rustc -p qvopenapi-http --release --target i686-pc-windows-gnu --features "disable-unwind" -- -C "panic=abort"

# Run using wine
wine target/i686-pc-windows-gnu/release/qvopenapi-http.exe
```

#### Windows
- LLVM 설치 필요 [링크](https://github.com/llvm/llvm-project/releases/tag/llvmorg-17.0.1)

```sh
# Setup rust cross-compilation
rustup target add i686-pc-windows-msvc

# Download dlls
scripts/download_dll.sh

# Build webserver
cargo build -p qvopenapi-http --release --target i686-pc-windows-msvc

# Run webserver
cargo run -p qvopenapi-http --release --target i686-pc-windows-msvc
```

### 인증서 복사
- 윈도우 하드디스크에 인증서 저장
- 윈도우에서 `C:\\Users\<윈도우 유저 이름>\AppData\LocalLow\NPKI` 폴더를 백업
- wine을 실행할 환경에서 `~/.wine/drive_c/users/<Wine 유저 이름>/AppData/LocalLow/NPKI` 폴더로 복사
  - ex: `cp -R NPKI ~/.wine/drive_c/users/crossover/AppData/LocalLow/NPKI`

### DLL 정상 작동 테스트
- 위 step을 모두 따라한 상태에서 `./run_example.sh` 실행
- `Enter QV_ID: ` 등이 출력되면 순서대로 나무증권 ID, 비밀번호, 인증서 비밀번호를 입력
  - 민감정보이므로 확실한 브랜치에서 실행하는지 확인할 것!!
- 로그에 자신의 ID 및 계좌 잔고가 정상적으로 출력되는지 확인
