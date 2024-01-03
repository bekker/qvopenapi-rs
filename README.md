# qvopenapi-rs
- NH투자증권에서 제공하는 QV Open API를 Rust에서 사용할 수 있도록 wrapping
- 자동으로 Windows 애플리케이션을 실행 및 이벤트를 관리
- REST API 형태로 호출할 수 있는 웹서버 애플리케이션
- Wine을 통해 리눅스에서 실행할 수 있는 Docker 이미지

## Current Status
- Rust 라이브러리 형태로 실행 가능
- 리눅스 서버에서 docker 컨테이너를 실행하여 HTTP로 통신 가능
- 계정 로그인 및 계좌 잔고 조회 가능

## 실행 환경
- Windows에서 native 지원
- macOS에서 wine-crossover를 사용하여 윈도우 애플리케이션 실행
- Linux에서 wine을 사용하여 윈도우 애플리케이션 실행 가능
- QV API가 윈도우 이벤트 기반으로 설계되었기 때문에 디스플레이가 없는 환경에서는 제대로 동작하지 않음
  - Docker 컨테이너에서는 `Xvbf`를 사용하여 가상의 디스플레이 환경으로 실행
  - `Dockerfile` 및 `docker/entrypoint.sh` 참고

## Example
```rust
// client 객체 생성
let future_client = QvOpenApiAsyncClient::new()?;

// ID, 비밀번호, 공동인증서 비밀번호를 이용해 계정 로그인
let connect_response = future_client.connect(
    AccountType::NAMUH,
    id.as_str(),
    password.as_str(),
    cert_password.as_str(),
).await?;
info!("connect response: {}", connect_response);

// 계좌 잔고 조회 (TR C8201)
// (상세 인터페이스는 NH투자증권의 QV API 자료 참고)
let query_response = future_client.query(C8201Request::new( 1, '1').into_raw()).await?;
info!("query response: {}", query_response);
```

## 실제 주문 및 실시간 TR을 지원할 계획은 없나요?
- 개인적인 잔고 조회 용도로 개발한 라이브러리라서 근시일 안에 지원 계획은 없으나 PR is welcome

## Modules
- `qvopenapi-sys`: `wmca.dll`을 `libloading`을 사용해서 DLL의 함수들을 호출할 수 있도록 작성
- `qvopenapi-binding`: QV API 예제 프로그램에서 사용하는 모델들을 rust에서 사용할 수 있도록 `bindgen`을 사용해서 자동 변환
- `qvopenapi`: `wmca.dll`은 윈도우 이벤트 기반으로 동작하므로 윈도우 및 기반 이벤트들을 자동으로 관리하고 주요 Tx 들을 별도 메소드로 제공
- `qvopenapi-async`: `qvopenapi`의 경우 콜백 기반으로 통신하기 때문에 TR ID 등을 관리하기가 어렵고 번거로움. Rust의 `future` 형태로 손쉽게 사용할 수 있도록 wrapping한 라이브러리
- `qvopenapi-http`: 다른 언어로 작성된 애플리케이션과 통신하기 편하도록 HTTP 프로토콜 제공. 내부적으로 `qvopenapi-async` 사용

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
- 위 step을 모두 따라한 상태에서 `scripts/run_example.sh` 실행
- `Enter QV_ID: ` 등이 출력되면 순서대로 나무증권 ID, 비밀번호, 인증서 비밀번호를 입력
  - 민감정보이므로 확실한 브랜치에서 실행하는지 확인할 것!!
- 로그에 자신의 ID 및 계좌 잔고가 정상적으로 출력되는지 확인

## Disclaimer
- 본 프로그램은 NH투자증권에서 제공하는 QV API 모듈을 기반으로 하며 해당 모듈 및 인터페이스 정보는 NH투자증권의 자산입니다.
- 프로그램 실행에 공동인증서 및 공동인증서 비밀번호가 필요하며 잔고 조회 / 주식 거래 기능은 민감한 정보를 다루고 있습니다.
- 프로그램 내부에 민감 정보를 저장하거나 외부로 유출하는 기능은 없으나 개인의 부주의로 인한 민감 정보 유출 혹은 금전적 손실에 대해서는 책임지지 않습니다.
