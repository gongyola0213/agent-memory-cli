# CLI Specification (v0.2 draft)

## Command Groups
- `doctor`
- `admin`
- `user`
- `identity`
- `scope`
- `schema`
- `dynamic`
- `ingest`
- `query`
- `state`

---

## user
Manage canonical users.

```bash
agent-memory-cli user create --name "Yongseong Kim"
agent-memory-cli user list
agent-memory-cli user show --uid <uid>
agent-memory-cli user update --uid <uid> --name "New Name"
agent-memory-cli user merge --from <from_uid> --to <to_uid>
agent-memory-cli user delete --uid <uid> --mode soft
agent-memory-cli user delete --uid <uid> --mode hard --force
agent-memory-cli user delete --uid <uid> --mode soft --dry-run
```

Delete guard policy (current):
- default mode is `soft`
- `hard` requires `--force`
- `hard` is allowed only when user status is `merged`

## identity
Map channel identities to canonical users.

```bash
agent-memory-cli identity link \
  --uid <uid> \
  --channel telegram \
  --channel-user-id 7992342261

agent-memory-cli identity resolve --channel telegram --channel-user-id 7992342261
agent-memory-cli identity unlink --channel telegram --channel-user-id 7992342261
```

Rules:
- unique(`channel`, `channel_user_id`)
- no auto-merge

## scope
Define memory boundaries.

```bash
agent-memory-cli scope create --id shared:couple --type shared
agent-memory-cli scope add-member --id shared:couple --uid <uid> --role member
agent-memory-cli scope list
agent-memory-cli scope members --id shared:couple
```

## schema
Register, inspect, and map dynamic schemas.

```bash
agent-memory-cli schema validate --file schema/restaurant-rating.json
agent-memory-cli schema register --file schema/restaurant-rating.json
agent-memory-cli schema list

# planned UX additions
agent-memory-cli schema show --schema-id restaurant.rating.v1 [--version 1]
agent-memory-cli schema template --schema-id restaurant.rating.v1 [--version 1]
agent-memory-cli schema map --format table|mermaid [--focus restaurant.rating.v1] [--depth 1]
```

Dynamic schema contract:
- Required top-level keys: `schema_id`, `version`, `class`, `fields`.
- `class` must be one of: `domain`, `user_context`.
- `fields` must be an array and each field must have a non-empty `name`.
- Duplicate field names are not allowed.
- `user_context` schemas require `refUserId`.
- `domain` schemas do not require `refUserId`.

Storage behavior:
- `schema register` writes metadata to `schema_registry`.
- `schema register` also creates physical table `dyn_<schema_id>_v<version>` (normalized).
- registration emits outbox event (`schema.registered`) for projection consumers.

## dynamic
Operate dynamic-schema records (planned group; foundation in progress).

```bash
agent-memory-cli dynamic upsert \
  --schema-id restaurant.rating.v1 \
  --version 1 \
  --record-id rec_001 \
  --entity-key restaurant_abc \
  --uid u_123 \
  --scope shared:couple \
  --file payload.json

# planned read helpers
agent-memory-cli dynamic get --schema-id restaurant.rating.v1 --record-id rec_001
agent-memory-cli dynamic list --schema-id restaurant.rating.v1 --uid u_123 --limit 20
```

Behavior targets:
- default to latest active version when `--version` omitted.
- strict schema validation for unknown fields/type mismatch.
- `user_context` requires uid linkage.

## ingest
Write fixed event-stream records (materializer path).

```bash
agent-memory-cli ingest event \
  --uid <uid> \
  --scope private:<uid> \
  --type meal.rated \
  --file event.json

agent-memory-cli ingest batch --file events.ndjson
```

Contract:
- append event
- run materializers
- commit atomically

## query
Read fast materialized outputs.

```bash
agent-memory-cli query latest --uid <uid> --scope private:<uid>
agent-memory-cli query metric --uid <uid> --scope private:<uid> --key invest_style
agent-memory-cli query topk --uid <uid> --scope private:<uid> --topic food_pref --limit 3
```

## state
Direct key-value CRUD for latest states.

```bash
agent-memory-cli state get --uid <uid> --scope shared:couple --key travel_food_style
agent-memory-cli state set --uid <uid> --scope shared:couple --key travel_food_style --value '{"spicy":0.7}'
agent-memory-cli state delete --uid <uid> --scope shared:couple --key travel_food_style
```

## admin
Operational maintenance.

```bash
agent-memory-cli admin migrate
agent-memory-cli admin reindex
agent-memory-cli admin compact
agent-memory-cli admin archive --month 2026-02
```

## doctor
Health check and environment diagnostics.

```bash
agent-memory-cli doctor
agent-memory-cli --json doctor
```
