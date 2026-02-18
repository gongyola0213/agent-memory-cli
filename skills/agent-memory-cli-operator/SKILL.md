---
name: agent-memory-cli-operator
description: Operate agent-memory-cli as a deterministic memory engine. Use when deciding whether to store interaction memory, emitting request/meal/expense events, querying top patterns/preferences, or running explicit migration before memory operations.
---

# Agent Memory CLI Operator

Use this skill to interact with `agent-memory-cli` safely and consistently.

## Core contract
- Treat CLI as deterministic engine.
- Keep interpretation/policy decisions in skill layer.
- Never auto-store everything.

## Storage policy (default)
- Default: do not store.
- Store only when one of these is true:
  1. Explicit user request (e.g., “remember this”, `/remember`)
  2. High-value durable preference/pattern
  3. Future decision impact is clear
- If uncertain: skip or record as `request.logged` with conservative pattern.

## Required preflight
Before first DB use on a path:

```bash
agent-memory-cli admin migrate --db <db-path>
```

If command returns schema-not-initialized, run migrate explicitly and retry.

## Event emission
Use normalized payloads.

### request pattern
```bash
agent-memory-cli --db <db> ingest event \
  --uid <uid> --scope <scope> \
  --type request.logged \
  --file <json-file> \
  --idempotency-key <optional-key>
```

Payload:
```json
{ "pattern": "restaurant_reco" }
```

### food preference
Payload:
```json
{ "cuisine": "korean" }
```
Type: `meal.rated`

### spending category
Payload:
```json
{ "category": "coffee" }
```
Type: `expense.logged`

## Query patterns
- Latest event:
  - `query latest --uid <uid> --scope <scope>`
- Ranked preferences/patterns:
  - `query topk --uid <uid> --scope <scope> --topic <topic> --limit <n>`
- Raw counters:
  - `query metric --uid <uid> --scope <scope> --key <metric-key>`
  - `query metric --uid <uid> --scope <scope> --prefix counter:<topic>:`

Topics:
- `food_pref`
- `spend_category`
- `request_pattern`

## Reliability rules
- Prefer idempotency keys for retriable writes.
- On validation failure, do not retry blindly; fix payload first.
- Keep payload minimal and deterministic.

## References
- Command cookbook: `references/commands.md`
