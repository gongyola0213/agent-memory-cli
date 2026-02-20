# CLI Specification (v0.1)

## Command Groups
- `user`
- `identity`
- `scope`
- `schema`
- `ingest`
- `query`
- `state`
- `admin`

---

## user
Manage canonical users.

```bash
agent-memory-cli user create --name "Yongseong Kim"
agent-memory-cli user list
agent-memory-cli user show --uid <uid>
agent-memory-cli user update --uid <uid> --name "New Name"
```

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
- re-link requires explicit `--force`

## scope
Define memory boundaries.

```bash
agent-memory-cli scope create --id shared:couple --type shared
agent-memory-cli scope add-member --id shared:couple --uid <uid> --role member
agent-memory-cli scope list
agent-memory-cli scope members --id shared:couple
```

## schema
Register and validate dynamic event schemas.

```bash
agent-memory-cli schema register --file schema/food.json
agent-memory-cli schema list
agent-memory-cli schema validate --file schema/food.json
```

Dynamic schema registration contract:
- Required top-level keys: `schema_id`, `version`, `class`, `fields`.
- `class` must be one of: `domain`, `user_context`.
- `fields` must be an array and each field must have a non-empty `name`.
- Duplicate field names are not allowed.
- `user_context` schemas require `refUserId`.
- `domain` schemas do not require `refUserId`.
- Missing `refUserId` must fail validation only for `user_context` schemas.
- Recommended common fields for user-context schemas: `refScopeId`, `sourceEventId`, `createdAt`, `updatedAt`.

## ingest
Write time-series events.

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
