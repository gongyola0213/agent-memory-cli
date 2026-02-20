# Agent Memory CLI Architecture (v0.1)

## Purpose
A local-first, personal-scale memory engine for agents. Designed for 1-2 users (e.g., couple/household), optimized for fast daily use, explainable recommendations, and low operational overhead.

## Core Principles
1. **User-centric identity**: Internal `uid` is canonical.
2. **Channel extensibility**: Per-channel identities map to one user (`1:N`).
3. **Event-sourced memory**: Append-only time-series events.
4. **Fast reads via materialization**: `state/metrics/topk` updated on write.
5. **RDB-first, Graph-derived**: SQLite is source of truth; graph is optional derived index.
6. **Local-first privacy**: Data stored locally; sensitive data minimized.

## Canonical Identity Model
- User key: `uid` (ULID/UUID)
- External identity: `(channel, channel_user_id)`
- Mapping: `users (1) -> (N) user_identities`

Benefits:
- Unified experience across Telegram/WhatsApp/Discord/etc.
- Stable memory continuity despite channel changes.
- Clean scope and permission controls by `uid`.

## Memory Scopes
- `private:<uid>`
- `shared:couple`
- (optional) `shared:household`

Default read order:
1. requestor private
2. couple shared (if allowed)
3. global/general knowledge

## Storage Strategy (Hot/Warm/Cold)
- **Hot (SQLite, last 30 days)**: real-time event ingest + serving layer.
- **Warm (SQLite summaries)**: weekly/monthly materialized summaries.
- **Cold (compressed archive files)**: monthly raw event archive (NDJSON/Parquet/Gzip).
- **Graph (optional)**: periodic derived relationship snapshots.

This mirrors “memory consolidation”: fresh events stay hot; long-term patterns become compact summaries.

## Data Planes
### 1) Source of Truth (SQLite)
- `events` append-only log
- normalized user/scope/identity tables
- transactional updates for derived state

### 2) Serving Plane (SQLite materialized)
- `state`: latest values
- `metrics`: aggregate values
- `topk`: ranked preferences/patterns

### 3) Optional Graph Plane (Cozo)
- relation-heavy traversal/ranking
- rebuilt from SQLite summaries/events if needed

## Ingestion Flow
Single transaction per event:
1. Validate schema/policy
2. Resolve identity -> `uid`
3. Insert into `events`
4. Run materializers (update `state/metrics/topk`)
5. Commit

If any step fails, rollback entire transaction.

## Initial Domain Materializers
1. `meal.rated` -> `topk(food_pref)`
2. `request.logged` -> repeat pattern metrics
3. `investment.updated` -> latest investment style + trend metric
4. `expense.logged` -> `topk(spend_category)`

## Dynamic Schema Contract (v0.2 direction)
For dynamic domain expansion (movie/game/performance/docs/email-style/etc), split schema classes:

1. **Domain schema (entity)**
   - Independent objects (e.g., place, city, restaurant, category, show).
   - Must be modelable without user linkage.
   - `refUserId` is **not required**.

2. **User-context schema (fact/edge)**
   - User-specific relations (e.g., liked, visited, rated, satisfied_with, recommended_to).
   - Must include `refUserId` (canonical `uid` reference).
   - Recommended fields: `refScopeId`, `sourceEventId`, `createdAt`, `updatedAt`.

Validation rule:
- Require `refUserId` only for user-context schema.
- Do not require `refUserId` for pure domain schema.

Storage rule (SSOT performance direction):
- On schema registration, create a physical SQLite table per schema version (`dyn_<schema_id>_v<version>` normalized).
- Keep schema_registry as metadata source-of-truth for type/version/validation.
- Use projection outbox for graph-index synchronization events.

Lifecycle rule:
- User merge/delete operations must process user-context dynamic tables by `refUserId`.
- Domain schemas remain independent; only relation tables are rewritten/relinked as needed.
- New dynamic schema modules must provide merge/delete handling hooks (or use shared default hook) before activation.

## Security & Governance
- Sensitive classes (finance/account/health): summary-first, minimal raw storage.
- External/irreversible actions require explicit confirmation.
- Correction/deletion must propagate across hot/warm/cold/graph layers.

## Non-goals (v0)
- Multi-tenant cloud service
- Distributed storage system
- Heavy infra beyond personal scale
