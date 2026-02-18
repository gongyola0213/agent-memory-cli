# Install

## Option A (recommended): Download release binary

1. Go to GitHub Releases.
2. Download the archive for your OS:
   - Linux: `x86_64-unknown-linux-gnu.tar.gz`
   - macOS: `x86_64-apple-darwin.tar.gz`
   - Windows: `x86_64-pc-windows-msvc.zip`
3. Extract and place `agent-memory-cli` (or `agent-memory-cli.exe`) on your PATH.

## Option B: Build locally

```bash
cargo build --release
```

Binary path:
- Unix: `target/release/agent-memory-cli`
- Windows: `target/release/agent-memory-cli.exe`

## First run

Run migration explicitly before use:

```bash
agent-memory-cli admin migrate --db <path-to-db>
```
