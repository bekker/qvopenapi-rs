# AGENTS.md — Instructions for AI Coding Agents

## ⚠️ MANDATORY: Read Before Any Code Operation

**Before reading or modifying any source code in this repository, you MUST first read [`ARCHITECTURE.md`](ARCHITECTURE.md) in its entirety.**

This project has non-obvious architectural constraints (Win32 message loops, DLL FFI, cross-compilation to i686, CP949 encoding, Wine compatibility) that make it easy to introduce subtle bugs if you lack context. The architecture document provides:

- Crate dependency graph and layering
- Per-file descriptions of every source file
- Threading model and message dispatch flow
- Platform constraints and build requirements
- Step-by-step guide for adding new TR (transaction) types
- Common pitfalls specific to this codebase

## Project Overview

`qvopenapi-rs` is a Rust wrapper for a proprietary Windows DLL (`wmca.dll`) from NH Investment & Securities (Korean brokerage). It provides:
- A Rust library with async/await support for brokerage operations
- An HTTP REST server for language-agnostic access
- Docker deployment via Wine for Linux servers

## Repository Structure

```
qvopenapi-rs/
├── qvopenapi-sys/         # Layer 0: Raw DLL function loading (libloading)
├── qvopenapi-bindings/    # Layer 0: C struct bindings (bindgen)
├── qvopenapi/             # Layer 1: Win32 window/message management + callback API
├── qvopenapi-async/       # Layer 2: Future-based async wrapper
├── qvopenapi-http/        # Layer 3: HTTP REST server (warp)
├── docker/                # Docker entrypoint
├── scripts/               # Build and run scripts
├── Dockerfile             # Multi-stage Wine-based Docker build
└── ARCHITECTURE.md        # ← READ THIS FIRST
```

## Critical Constraints

1. **Target**: All crates compile to `i686-pc-windows-gnu` or `i686-pc-windows-msvc` (32-bit Windows). There is no native Linux/macOS build.
2. **Feature flag**: `disable-unwind` is required on non-MSVC platforms (provides mock `_Unwind_Resume` for MinGW).
3. **Single-threaded runtime**: The HTTP server uses `tokio::runtime::Builder::new_current_thread()` intentionally — the Win32 message pump must run on the main thread.
4. **No test suite**: There are no unit or integration tests. Verification requires brokerage credentials and network access.
5. **C struct encoding**: All DLL string data is CP949-encoded, fixed-width char arrays (not null-terminated).

## Coding Conventions

- **Language**: Code and comments are in English, but domain-specific terms (TR codes, error messages, API field names) are in Korean.
- **Error handling**: Use `QvOpenApiError` enum (16 variants) with `Result<T, QvOpenApiError>`. Use `From` trait conversions for `?` operator.
- **Models**: TR request/response types live in `qvopenapi/src/models/query/`. Each TR code gets its own file.
- **Serialization**: Response data is converted from C structs → Rust structs → `serde_json::Value`.
- **HTTP framework**: `warp` (not actix-web). Routes are composed via `.or()` filter combinator.

## When Making Changes

- **Adding a new TR type**: Follow the step-by-step guide in Section 9 of `ARCHITECTURE.md`.
- **Modifying DLL interaction**: Understand the threading model (Section 4.3 of `ARCHITECTURE.md`) — all DLL calls must happen on the window thread.
- **Modifying HTTP endpoints**: See `qvopenapi-http/src/routes/` — each endpoint is a warp filter function.
- **Modifying Docker**: See `Dockerfile` and `docker/entrypoint.sh`. Xvfb + xfce4-session are required for Win32 message dispatch under Wine.
