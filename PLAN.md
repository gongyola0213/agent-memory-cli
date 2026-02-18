# Project Plan (v0.1)

## 1) Product Scope

Build a general-purpose local memory CLI for agents to store time-series events and query personalized state quickly.

### Primary Use Cases

- Favorite food preference tracking
- Repeated request pattern detection
- Investment style trend tracking
- Spending category preference tracking

## 2) Architecture

### Core Tables (fixed)

- `events` (append-only)
- `entities`
- `edges`
- `state` (latest values)
- `metrics` (materialized aggregates)
- `topk` (pre-ranked results)
- `schema_registry`

### Design Pattern

- Write path: append event + update materialized state in one transaction
- Read path: serve from state/metrics/topk for O(1)-like access

## 3) CLI Design

```bash
memcli schema register --file schema.json
memcli ingest event --file event.json
memcli query latest --user me
memcli query metric --user me --key invest_style
memcli query topk --user me --topic food_pref
```

## 4) Milestones

### M1: Foundation
- Rust CLI scaffold (clap)
- SQLite bootstrap/migrations
- schema registry commands

### M2: Ingestion & Materialization
- event ingestion
- transactional materialized updates
- basic query commands

### M3: Packaging
- GitHub Actions cross-platform build
- Release artifacts upload

## 5) Non-Goals (for now)

- Multi-tenant cloud service
- Heavy distributed storage
- Real-time streaming infra

## 6) Security Baseline

- Local DB file permissions
- Avoid plaintext secret logging
- Principle of least privilege for CI tokens
