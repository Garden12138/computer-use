# Native helpers (Rust)

Workspace for Windows and Linux helpers. macOS stays in `helper/macos` (Swift).

```bash
# Any OS: protocol/coords/PNG tests (no GUI)
cargo test -p computer-use-core

# Windows
cargo build -p computer-use-windows --release
# binary: target/release/computer-use-helper-windows.exe

# Linux
cargo build -p computer-use-linux --release
# binary: target/release/computer-use-helper-linux
```

`scripts/build-helper-native.sh` wraps that and copies the binary to `dist/`.
