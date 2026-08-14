#!/usr/bin/env bash
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT/helper/native"
cargo test -p computer-use-core
case "$(uname -s)" in
  Linux)
    cargo build -p computer-use-linux --release
    mkdir -p "$ROOT/dist"
    cp "$ROOT/helper/native/target/release/computer-use-helper-linux" "$ROOT/dist/computer-use-helper"
    echo "built $ROOT/dist/computer-use-helper"
    ;;
  MINGW*|MSYS*|CYGWIN*)
    cargo build -p computer-use-windows --release
    mkdir -p "$ROOT/dist"
    cp "$ROOT/helper/native/target/release/computer-use-helper-windows.exe" "$ROOT/dist/computer-use-helper.exe"
    echo "built $ROOT/dist/computer-use-helper.exe"
    ;;
  *)
    echo "On this OS only computer-use-core tests ran."
    echo "Build the platform helper on Windows: cargo build -p computer-use-windows --release"
    echo "Build the platform helper on Linux: cargo build -p computer-use-linux --release"
    ;;
esac
