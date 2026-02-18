# gongyola-agent-memory-cli

A lightweight local-first memory engine CLI for personal agents (OpenClaw/Claude/Codex style), focused on:

- Time-series interaction logging
- Dynamic schema registration
- Materialized state for fast reads
- Graph-friendly relations over local DB

## Goals

1. Local-first, single-user/couple scale
2. Fast ingest and O(1)-like reads for precomputed metrics
3. Generic schema model for multiple agents/domains
4. Simple deployment via release binaries

## Planned Stack

- Rust (CLI)
- SQLite (event store + materialized state)
- Optional Cozo integration for advanced graph-style queries

## Initial MVP Scope

- `schema register|list|validate`
- `ingest event`
- `query latest|metric|topk`
- `state get|set|delete`

## Development

```bash
make lint      # clippy with warnings denied
make typecheck # cargo check
make test
```

## Status

Scaffolding phase.
