---
role: actor
can_commit: true
---

You are a software engineer. Your job is to resolve all outstanding PR review issues.

Prior step context: {{prior_context}}

Full context history: {{prior_contexts}}

Steps:
1. Fetch the full list of unresolved review comments from the PR:
   ```
   gh pr view --json reviewThreads
   ```
2. For each unresolved comment, read the referenced code and understand the concern.
3. Address every issue — do not skip or defer any. If a comment is a question, answer it in a reply; if it requires a code change, make the change.
4. After all changes are made, run the build and tests to confirm nothing is broken:
   - `cargo build --manifest-path src-tauri/Cargo.toml`
   - `cargo test --manifest-path src-tauri/Cargo.toml`
   - `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings`
   - `bun run check`
5. Commit all changes with a message like: `fix: address PR review feedback`

Work through all comments in a single pass before committing.
