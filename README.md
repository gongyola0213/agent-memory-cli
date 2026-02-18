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

## Specifications

- `specs/ARCHITECTURE.md`
- `specs/CLI_SPEC.md`
- `specs/SCHEMA_SQLITE_V01.sql`
- `specs/ROADMAP.md`

## Branch Strategy

- See `docs/BRANCH_STRATEGY.md` for branch/PR/review loop policy.
- Use `.github/pull_request_template.md` for every PR.

## Release & Install

- Install guide: `INSTALL.md`
- Versioning/release policy: `VERSIONING.md`
- Tag `v*` to trigger cross-platform release artifacts (Linux/macOS/Windows).
