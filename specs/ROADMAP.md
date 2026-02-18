# Implementation Roadmap (v0)

## Phase 1: Foundation (Week 1)
- [ ] Add migration runner and initialize SQLite schema
- [ ] Implement `user`, `identity`, `scope` commands
- [ ] Implement `schema register/list/validate`
- [ ] Add policy loading from workspace policy/profile files

## Phase 2: Ingestion + Serving (Week 2)
- [ ] Implement `ingest event` atomic transaction path
- [ ] Implement first materializers (`meal.rated`, `expense.logged`)
- [ ] Implement `query latest/metric/topk`
- [ ] Implement `state` CRUD

## Phase 3: Ops + Lifecycle (Week 3)
- [ ] Add monthly archive command (`admin archive`)
- [ ] Add summary compaction (`weekly/monthly`) tables
- [ ] Add reindex/rebuild command for derived tables
- [ ] Add export/import for portability

## Phase 4: Optional Graph Layer
- [ ] Define SQLite -> Cozo derived sync contract
- [ ] Add snapshot sync job
- [ ] Add graph-backed recommendation query path

## Success Criteria
- fast local reads from materialized tables
- clear user identity resolution across channels
- predictable memory scope boundaries
- low-ops personal deployment
