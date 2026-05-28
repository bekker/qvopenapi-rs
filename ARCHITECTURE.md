# ARCHITECTURE.md — qvopenapi-rs

> Last updated: 2026-05-28

> **Audience**: AI coding agents. This document is designed for fast, accurate context-loading before code reading or modification tasks. Human readability is secondary.

## 1. Project Purpose

`qvopenapi-rs` wraps a **proprietary Windows DLL** (`wmca.dll`) provided by NH Investment & Securities (NH투자증권, a Korean brokerage) into a Rust library stack and HTTP server. The DLL exposes a Win32 message-based API for stock trading operations (login, balance queries, order placement). This project makes the DLL usable from:
- Rust code (library crate)
- Any language via HTTP (server crate)
- Linux servers via Wine + Docker

## 2. Workspace Layout (Cargo workspace)

```
qvopenapi-rs/                  # Workspace root
├── Cargo.toml                 # Workspace manifest (members listed below)
├── Dockerfile                 # Multi-stage Docker build (Wine-based)
├── docker/entrypoint.sh       # Container entrypoint: Xvfb → xfce4-session → wine exe
├── scripts/                   # Shell scripts for build, run, DLL download
│
├── qvopenapi-sys/             # Layer 0: Raw DLL function loading
├── qvopenapi-bindings/        # Layer 0: C struct bindings (bindgen)
├── qvopenapi/                 # Layer 1: Win32 window/message management + callback API
├── qvopenapi-async/           # Layer 2: Future-based async wrapper
└── qvopenapi-http/            # Layer 3: HTTP REST server (warp)
```

## 3. Crate Dependency Graph

```
qvopenapi-sys ─────┐
                    ├──→ qvopenapi ──→ qvopenapi-async ──→ qvopenapi-http
qvopenapi-bindings─┘
```

Each higher layer depends on all lower layers. The dependency is strictly linear — no circular dependencies.

## 4. Crate Details

---

### 4.1 `qvopenapi-sys` (Layer 0 — DLL Loading)

**Path**: `qvopenapi-sys/`
**Purpose**: Dynamically load `wmca.dll` at runtime using `libloading` and expose its raw C function pointers as safe Rust function calls.

#### Key Files

| File | Description |
|------|-------------|
| `src/lib.rs` | Single-file crate. Uses `libloading::Library` to load `wmca.dll` and resolves 20 function pointers via `bind_lib()`. Defines `WmcaLib` struct holding the `Library` handle + 20 `RawSymbol` fields. Has `disable-unwind` feature for MinGW compat. |

#### DLL Functions Loaded (from `wmca.dll`)
- `wmcaLoad` / `wmcaFree` — Initialize / cleanup the API
- `wmcaConnect` / `wmcaDisconnect` — Login / logout
- `wmcaIsConnected` — Check connection status
- `wmcaSetServer` / `wmcaSetPort` — Set server address / port
- `wmcaQuery` / `wmcaTransact` / `wmcaRequest` — Send TR queries
- `wmcaAttach` / `wmcaDetach` / `wmcaDetachWindow` / `wmcaDetachAll` — Real-time data subscribe/unsubscribe
- `wmcaSetOption` — Set API options
- `wmcaSetAccountIndexPwd` / `wmcaSetOrderPwd` / `wmcaSetHashPwd` / `wmcaSetAccountNoPwd` / `wmcaSetAccountNoByIndex` — Password/account management

#### Critical Invariant
The DLL is an **i686 (32-bit) Windows binary**. All builds must target `i686-pc-windows-gnu` or `i686-pc-windows-msvc`. The entire project compiles to a 32-bit Windows executable.

---

### 4.2 `qvopenapi-bindings` (Layer 0 — C Struct Bindings)

**Path**: `qvopenapi-bindings/`
**Purpose**: Use `bindgen` to auto-generate Rust FFI types from C header files provided in the QV API SDK.

#### Key Files

| File | Description |
|------|-------------|
| `build.rs` | Runs `bindgen` on `src/bindings/bindings.h` (which includes `trio_inv.h` + `trio_ord.h`). Generates Rust FFI output to `$OUT_DIR/bindings.rs`. |
| `src/lib.rs` | Re-exports `bindings::*`. Suppresses naming/dead-code warnings. |
| `src/bindings/mod.rs` | Includes auto-generated bindings + 6 hand-written `#[repr(C)]` structs for the DLL's callback messaging protocol: `OutDataBlock<T>`, `ReceivedData<T>`, `MessageHeader`, `LoginBlock`, `LoginInfo`, `AccountInfo`. |
| `src/bindings/bindings.h` | Master header that includes `trio_inv.h` and `trio_ord.h`. |
| `src/bindings/trio_inv.h` | ~3241 lines. Investment/inquiry data structures — stock quotes, futures/options, sector indices, ETF, real-time streaming structs. |
| `src/bindings/trio_ord.h` | ~746 lines. Order/trading data structures — stock/ELW orders, futures/options orders, balance inquiries, real-time execution/status notifications. |

#### Generated Types
Hundreds of `#[repr(C, packed)]` structs following the pattern `TxxxxInBlock` (input), `TxxxxOutBlock` (output). All fields are **fixed-width `c_char` arrays** (NOT null-terminated — use separator chars in Korean brokerage fixed-width convention).

#### Build Dependency
The header files are bundled in `src/bindings/` within the crate itself (not downloaded separately).

---

### 4.3 `qvopenapi` (Layer 1 — Core Library)

**Path**: `qvopenapi/`
**Purpose**: Manages the Win32 window lifecycle, message loop, and DLL callback dispatching. Provides a callback-based Rust API for connecting and querying the brokerage.

#### Key Files

| File | Description |
|------|-------------|
| `src/lib.rs` | Module declarations, re-exports public API. Defines `AbstractQvOpenApiClient` trait. Re-exports: `QvOpenApiClient`, `QvOpenApiClientMessageHandler`, `QvOpenApiRequest`, `WindowHelper`, `WindowStatus`, `init`, `is_connected`, `set_port`, `set_server`. |
| `src/client.rs` | **Central type: `QvOpenApiClient`** (implements `AbstractQvOpenApiClient`). Wraps `Arc<QvOpenApiClientMessageHandler>`. `QvOpenApiClientMessageHandler` holds hwnd (`RwLock<Option<isize>>`), 8 boxed callback closures (`QvOpenApiClientMessageCallbacks`), and a request queue (`Mutex<VecDeque>`). `on_wmca_msg(wparam, lparam)` dispatches messages by matching wparam against `CA_*` constants. |
| `src/wmca_lib.rs` | FFI wrapper around `wmca.dll` via `qvopenapi-sys`. Uses `OnceCell<WmcaLib>` for lazy singleton DLL binding. Provides: `init()`, `is_connected()`, `set_server()`, `set_port()`, `connect()`, `query()`, `disconnect()`, `set_account_index_pwd()`. Converts `AccountType` to DLL media/user type codes. |
| `src/error.rs` | `QvOpenApiError` custom error enum (16 variants) using `custom_error!` macro. Derives `Clone, Serialize`. Implements `From` for `libloading::Error`, `windows::core::Error`, `chrono::ParseError`, `serde_json::Error`. |
| `src/utils/mod.rs` | Utility functions: `from_cp949()` / `from_cp949_ptr()` (CP949 Korean encoding → String), `parse_string()`, `parse_number()`, `parse_ratio()`, `parse_ratio_str()`. `SEOUL_TZ` constant (UTC+9). |
| `src/window_mgr/mod.rs` | `WindowHelper` struct (manages window lifecycle: hwnd, `WindowStatus` enum, thread handle). `run()` creates window async and returns hwnd. `destroy()` tears down. Conditional compilation: `#[cfg(target_os = "windows")]` → `window_mgr_win32`, else → `window_mgr_mock`. |
| `src/window_mgr/message_const.rs` | Win32 message constants. `WM_WMCAEVENT = WM_USER + 8400` (main DLL event). `CA_CUSTOM_EXECUTE_POSTED_COMMAND = WM_USER + 8410` (custom: triggers request queue drain). `CA_CONNECTED/DISCONNECTED/SOCKETERROR/RECEIVEDATA/RECEIVESISE/RECEIVEMESSAGE/RECEIVECOMPLETE/RECEIVEERROR`. |
| `src/window_mgr/window_mgr_win32.rs` | Win32 implementation. Registers `WNDCLASSW` (class name `"qvopenapi"`), creates a 400×300 window with `wndproc`. `wndproc` handles `WM_WMCAEVENT` by looking up handler from a global `RwLock<HashMap<isize, Arc<QvOpenApiClientMessageHandler>>>` (`MESSAGE_HANDLER_MAP_LOCK`). Standard `GetMessageW`/`TranslateMessage`/`DispatchMessageW` loop. |
| `src/window_mgr/window_mgr_mock.rs` | Non-Windows stub. All functions call `unimplemented!()`. Allows compilation on non-Windows platforms but not execution. |
| `src/models/mod.rs` | Module declarations for models. Re-exports key types. |
| `src/models/connect.rs` | `ConnectRequest` (implements `QvOpenApiRequest`), `ConnectResponse`, `AccountInfoResponse`. `parse_connect(lparam)` reads from `LoginBlock` C struct with CP949 decoding. Defines `TR_INDEX_CONNECT = 1`. |
| `src/models/message.rs` | `MessageResponse`, `ErrorResponse` structs. `parse_message()`, `parse_complete()`, `parse_error()` functions — parse from `OutDataBlock<T>` raw pointers. |
| `src/models/query/mod.rs` | `DataResponse` struct (`tr_index`, `block_name`, `block_data: Value`). `parse_data()`, `parse_sise()`. `RawQueryRequest<T>` generic wrapper. `DisconnectRequest`. `parse_block()` dispatches by `block_name` string. |
| `src/models/query/c8201.rs` | TR `C8201` (계좌 잔고조회): `C8201Request` → `into_raw()` → `Arc<RawQueryRequest<Tc8201InBlock>>`. `parse_c8201_response()` (29 fields), `parse_c8201_response1_array()` (17 fields per holding). Block names: `c8201OutBlock`, `c8201OutBlock1`. |

#### Threading Model
1. `WindowHelper::run()` spawns a **dedicated OS thread** that creates the window and enters the Win32 message loop.
2. The DLL communicates via `WM_WMCAEVENT` (`WM_USER + 8400`) Win32 messages to the window.
3. `wndproc` looks up the handler from a global `MESSAGE_HANDLER_MAP_LOCK` (`RwLock<HashMap<isize, Arc<QvOpenApiClientMessageHandler>>>`), then calls `on_wmca_msg(wparam, lparam)`.
4. User code calls `client.query()` from any thread → request pushed to `VecDeque` → `PostMessageA(CA_CUSTOM_EXECUTE_POSTED_COMMAND)` → window thread drains queue → `call_lib()` on window thread.
5. All DLL calls happen on the window thread. Callbacks invoke user-registered closures (protected by `Mutex`).

#### Key Traits
- **`QvOpenApiRequest`** (`Send + Sync`): `before_post()`, `call_lib(tr_index, hwnd)`, `get_tr_code()`. Implemented by `ConnectRequest`, `RawQueryRequest<T>`, `DisconnectRequest`.
- **`AbstractQvOpenApiClient`**: 8 callback setters (`on_connect`, `on_disconnect`, `on_socket_error`, `on_data`, `on_sise`, `on_message`, `on_complete`, `on_error`) + `connect()`, `disconnect()`, `query()` methods.

#### Encoding
All string data from the DLL is **CP949 (Korean Windows encoding)**. The `utils::from_cp949()` function handles conversion to UTF-8.

#### Win32 Message Constants (WM_USER offsets)
| Constant | Meaning |
|----------|---------|
| `CA_WMCAEVENT` (WM_USER+8400) | Primary DLL event message |
| `CA_CONNECTED` (WM_USER+110) | Connection established |
| `CA_DISCONNECTED` (WM_USER+120) | Connection lost |
| `CA_SOCKETERROR` (WM_USER+130) | Socket error |
| `CA_RECEIVEDATA` (WM_USER+210) | Query response data received |
| `CA_RECEIVESISE` (WM_USER+220) | Real-time market data |
| `CA_RECEIVEMESSAGE` (WM_USER+230) | System/server message |
| `CA_RECEIVECOMPLETE` (WM_USER+240) | Operation complete |
| `CA_RECEIVEERROR` (WM_USER+250) | Error response |

#### Example
`examples/connect.rs` — Interactive CLI that prompts for credentials, connects, queries C8201 (balance), and prints results.

---

### 4.4 `qvopenapi-async` (Layer 2 — Async Wrapper)

**Path**: `qvopenapi-async/`
**Purpose**: Wraps the callback-based `qvopenapi` client into `async/await` futures using a **manual `Future` implementation** with `Waker`-based signaling. Manages TR request IDs to match responses to awaiting futures.

#### Key Files

| File | Description |
|------|-------------|
| `src/lib.rs` | Module declarations. Re-exports `QvOpenApiAsyncClient`, `qvopenapi::error`, `qvopenapi::models`. |
| `src/client.rs` | **Central type: `QvOpenApiAsyncClient`**. Wraps `Arc<dyn AbstractQvOpenApiClient>`. For each operation, creates a `TrContext` with a `Mutex<TrContextStatus>` containing a `Waker` slot, stores it in `HashMap<i32, Arc<TrContext>>` keyed by TR index, calls the underlying client, and returns a `TrFuture`. Manages 7 callback registrations, TR index allocation (3–255 round-robin), and a background timeout-check thread. |
| `src/context.rs` | `TrContext` (per-request state), `TrContextStatus` (result accumulator + `Waker`), `TrFuture` (implements `std::future::Future<Output = Result<Value, QvOpenApiError>>`), `TrType` enum (`CONNECT`, `QUERY`). |

#### Async Pattern
```
User calls async_client.query(request).await
  → TrContext created with Mutex<TrContextStatus> (contains Waker slot)
  → Context stored in tr_context_map[tr_index]
  → underlying delegate.query() called (fires DLL call)
  → DLL response arrives → callback → context.on_data() accumulates blocks
  → DLL complete callback → context.on_complete() → set_done() → waker.wake()
  → TrFuture::poll() returns Poll::Ready(Ok(json_value))
```

#### TR Index Management
- `TR_INDEX_CONNECT = 1` — fixed index for connect operations
- Query indices: auto-allocated 3–255 (round-robin via `get_next_tr_index()`)
- Index 2 is unused (gap)
- Protected by `Mutex<i32>`, no collision check at allocation time

#### Timeout Handling
A background thread runs every 100ms, scanning for contexts older than 10 seconds (`DEFAULT_TIMEOUT`). Expired contexts receive `RequestTimeoutError` and are removed from the map. Thread is stopped via `is_dropping` flag set in `Drop` impl.

#### Output Format
The resolved `Value` is a JSON object: `{ "result": { ... }, "messages": [...], "error_type": ..., "errors": [...] }`. Even some error cases return inside `Ok(Value)` with `error_type` populated.

#### Feature Flags
- `disable-unwind`: Required on non-Windows platforms. Forwards to `qvopenapi/disable-unwind`. Provides a mock `_Unwind_Resume` symbol to work around missing DWARF unwind support in i686-pc-windows-gnu MinGW toolchain.

#### Example
`examples/connect_async.rs` — Same as basic example but using `async/await` with `tokio::main`.

---

### 4.5 `qvopenapi-http` (Layer 3 — HTTP Server)

**Path**: `qvopenapi-http/`
**Purpose**: REST API server using **`warp`**. Exposes brokerage operations as HTTP endpoints. Designed for deployment in Docker containers.

#### Key Files

| File | Description |
|------|-------------|
| `src/main.rs` | Entry point. Creates single-threaded Tokio runtime, creates `QvOpenApiAsyncClient` wrapped in `Arc`, starts `warp` server on `0.0.0.0:18000`. |
| `src/routes/mod.rs` | Combines all route filters using warp's `.or()` combinator. |
| `src/routes/connect.rs` | `POST /connect` — Login endpoint. JSON body: `ConnectRequest` (`account_type`, `id`, `password`, `cert_password`). |
| `src/routes/query.rs` | `POST /query/c8201` — Balance query endpoint. JSON body: `C8201Request`. |
| `src/routes/disconnect.rs` | `POST /disconnect` — Logout endpoint (no body). |
| `src/routes/connect_info.rs` | `GET /connect-info` — Returns cached connection info. |
| `src/error.rs` | Converts `QvOpenApiError` into HTTP responses (400 for `AlreadyConnectedError`, 500 for others). |
| `src/response.rs` | `HttpMessageResponse` — generic JSON response struct with `message` field. |

#### HTTP API Summary

| Method | Path | Purpose | Request Body |
|--------|------|---------|-------------|
| `POST` | `/connect` | Login to brokerage | `{ "account_type": "NAMUH", "id": "...", "password": "...", "cert_password": "..." }` |
| `GET` | `/connect-info` | Get cached connection info | (none) |
| `POST` | `/query/c8201` | Account balance query | `C8201Request` JSON |
| `POST` | `/disconnect` | Logout | (none) |

**Note**: All handlers return `Result<impl Reply, Infallible>` — errors are converted to JSON responses inline, never propagated as warp rejections.

#### Server Configuration
- Listens on `0.0.0.0:18000`
- Single-threaded Tokio runtime (`new_current_thread()`) — required because the underlying Win32 message pump must run on the main thread
- Single `QvOpenApiAsyncClient` instance in `Arc`, shared across all requests
- No authentication layer (relies on network-level security)

---

## 5. Docker & Deployment Architecture

### Container Stack
```
Docker Container (scottyhardy/docker-wine:stable-8.0)
├── Xvfb :99              ← Virtual framebuffer (headless X11 display)
├── xfce4-session          ← Window manager (required for Win32 message dispatch)
├── Wine                   ← Windows compatibility layer
│   └── qvopenapi-http.exe ← Rust HTTP server (i686 Windows binary)
│       └── wmca.dll       ← NH Securities proprietary DLL
```

### Why Xvfb + Window Manager?
The DLL (`wmca.dll`) uses Win32 window messages (`PostMessageW`, `GetMessageW`) for async callback delivery. This requires:
1. A display server (Xvfb provides a virtual one)
2. A window manager (xfce4-session) for proper message dispatch

Without these, the DLL's message-based callbacks fail silently.

### Docker Build (Multi-stage)
1. **setup**: Base Wine image, Korean locale/fonts, user creation
2. **dll**: Downloads `openapi.qv.zip` from NH Securities, extracts DLL files
3. **final**: Copies pre-built `qvopenapi-http.exe` + DLLs, sets entrypoint

### NPKI Certificate
The brokerage requires a Korean digital certificate (공동인증서) stored at:
- Container: `/namu/NPKI` (volume mount) → symlinked to Wine's `AppData/LocalLow/NPKI`
- Windows: `C:\Users\<user>\AppData\LocalLow\NPKI`

---

## 6. Build & Cross-Compilation

### Target Architecture
**Mandatory**: `i686-pc-windows-gnu` (Linux/macOS cross-compile) or `i686-pc-windows-msvc` (native Windows). The DLL is 32-bit only.

### Build Prerequisites
| Platform | Requirements |
|----------|-------------|
| macOS | `rustup target add i686-pc-windows-gnu`, `brew install mingw-w64 llvm`, wine-crossover |
| Windows | `rustup target add i686-pc-windows-msvc`, LLVM 17+ |
| Linux | MinGW toolchain, Wine (for testing) |

### DLL Download
`scripts/download_dll.sh` — Downloads `openapi.qv.zip` from `https://download.nhqv.com/download/iflgtrading/openapi.qv.zip`, extracts to `dlls/` directory. This provides:
- `wmca.dll` (runtime dependency)
- `include/*.h` (build-time dependency for `bindgen`)

### Feature Flag: `disable-unwind`
Required on non-MSVC targets. Defines a dummy `_Unwind_Resume` symbol to avoid linker errors from missing DWARF exception handling in MinGW i686. Also requires `panic = abort` in the build profile.

---

## 7. Scripts Reference

| Script | Purpose |
|--------|---------|
| `scripts/download_dll.sh` | Download + extract DLLs and headers from NH Securities |
| `scripts/run_example.sh` | Build & run basic sync example (macOS cross-compile) |
| `scripts/run_example_async.sh` | Build & run async example (macOS cross-compile) |
| `scripts/run_example_win.sh` | Build & run sync example (Windows native) |
| `scripts/run_example_async_win.sh` | Build & run async example (Windows native) |
| `scripts/run_http.sh` | Build & run HTTP server (macOS, via Wine) |
| `scripts/run_http_docker.sh` | Build for Docker + `docker build` + `docker run` |
| `scripts/run_http_win.sh` | Build & run HTTP server (Windows native) |

---

## 8. Key Patterns & Invariants

1. **All crates produce i686 Windows binaries.** There is no native Linux/macOS runtime — everything runs through Wine.
2. **The DLL is callback-based via Win32 messages.** This fundamentally shapes the architecture: a hidden window + message loop is always needed.
3. **Thread safety is achieved via message marshaling.** DLL callbacks arrive as Win32 messages on the window thread. User requests are queued via `VecDeque` + `PostMessageA` to ensure DLL calls happen on the window thread. Futures use `Waker`-based signaling to bridge to async callers.
4. **TR codes identify transaction types.** `C8201` = balance query. The system is extensible — new TR types require: (a) C struct bindings in `qvopenapi-bindings`, (b) Request/Response models in `qvopenapi/src/models/query/`, (c) route handler in `qvopenapi-http`.
5. **Error codes are DLL-specific numeric values.** See `qvopenapi/src/error_code.rs` for the full mapping.
6. **No test suite exists.** The project has no unit or integration tests. Testing requires actual brokerage credentials and network access to NH Securities' servers.
7. **The project currently supports only balance query (C8201).** Order placement and real-time data streaming are not implemented but the C header structs for orders (`trio_ord.h`) already exist in bindings.
8. **Single-threaded Tokio runtime is intentional.** The Win32 message pump must run on the main thread.

---

## 9. Adding a New TR (Transaction) Type — Step by Step

1. **Check if the C struct already exists** in `qvopenapi-bindings/src/bindings/trio_inv.h` or `trio_ord.h`. Many TR types have bindings but no Rust model yet.
2. **If the header is missing**, add the struct definition to the appropriate `.h` file in `qvopenapi-bindings/src/bindings/`.
3. **Create request/response models** in `qvopenapi/src/models/query/<tr_code>.rs`:
   - Define `<TrCode>Request` struct implementing `into_raw() -> RawQueryRequest`
   - Define `<TrCode>Response` struct with a `from_query_response()` parser
4. **Register the module** in `qvopenapi/src/models/query/mod.rs` — add to `parse_block()` match.
5. **Add HTTP route filter** in `qvopenapi-http/src/routes/` (new file or extend `query.rs`) and register in `routes/mod.rs`.
6. **Rebuild** with the appropriate cross-compile target.

---

## 10. Common Pitfalls for AI Agents

- **Do NOT attempt to compile or test on macOS/Linux natively.** The binary target is always Windows i686. Use cross-compilation.
- **Do NOT remove `disable-unwind` feature** when building on non-MSVC platforms — it will cause linker errors.
- **C struct layouts are `#[repr(C, packed)]`** — be careful with alignment and padding when adding new TR types.
- **String fields in C structs are fixed-width `c_char` arrays**, NOT null-terminated in many cases. Use `utils::from_cp949()` and related functions for conversion.
- **The `dlls/` directory is gitignored.** It must be populated by running `scripts/download_dll.sh` before building. However, the C header files for bindings are in `qvopenapi-bindings/src/bindings/` (not in `dlls/`).
- **The project has no tests.** Any code changes should be verified by building successfully and, if possible, running with actual credentials.
- **Single-threaded Tokio runtime is required.** Do not change `new_current_thread()` to `new_multi_thread()` in `qvopenapi-http`.
