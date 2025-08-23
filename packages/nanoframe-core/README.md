# nanoframe-core

Rust core for Nanoframe. Provides a JSON-RPC 2.0 API over stdio and manages system webviews (wry) and windows (tao).

Build:

```
cargo build -r
```

Run dev (from repo root, used by Node package when NAN_OF_DEV=1):

```
cargo run --release --bin nanoframe-core
```