#!/usr/bin/env bash
set -euo pipefail

cargo fmt --all --manifest-path src-tauri/Cargo.toml
if ! git diff --quiet; then
  git add -A
  git commit -m "style: cargo fmt"
fi
