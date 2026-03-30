#!/usr/bin/env bash
set -uo pipefail

ERRORS=0

# Run clippy on the Tauri Rust backend
cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings 2>&1 || ERRORS=1
cargo fmt --all --check --manifest-path src-tauri/Cargo.toml 2>&1 || ERRORS=1

# Run Svelte/TypeScript type checking
bun run check 2>&1 || ERRORS=1

if [ "$ERRORS" -eq 1 ]; then
  cat <<'EOF'
<<<CONDUCTOR_OUTPUT>>>
{"markers": ["has_lint_errors"], "context": "Lint errors found"}
<<<END_CONDUCTOR_OUTPUT>>>
EOF
else
  cat <<'EOF'
<<<CONDUCTOR_OUTPUT>>>
{"markers": [], "context": "All lint checks passed"}
<<<END_CONDUCTOR_OUTPUT>>>
EOF
fi
