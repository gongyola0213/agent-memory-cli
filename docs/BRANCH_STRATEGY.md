# Branch Strategy & Review Loop

## Goal
Keep changes safe, reviewable, and consensus-driven before merge.

## Standard Flow
1. Create a branch
2. Implement work on that branch
3. Open PR into `main`
4. Perform review
5. Run **RALPH loop** until consensus
6. Notify owner, then merge

## Branch Naming
- `feat/<short-topic>`
- `fix/<short-topic>`
- `docs/<short-topic>`
- `chore/<short-topic>`
- `refactor/<short-topic>`
- `test/<short-topic>`

Examples:
- `feat/user-identity-linking`
- `docs/memory-governance-v1`

## PR Rules
- No direct pushes to `main` for feature work.
- One PR = one coherent change.
- Include:
  - what changed
  - why
  - test plan
  - risks/rollback

## RALPH Loop (Review Until Agreement)
Repeat until all required reviewers agree:
1. **R**aise feedback
2. **A**ction fixes
3. **L**og rationale for non-adopted comments
4. **P**ublish update (new commit)
5. **H**and back for re-review

Exit criteria:
- Required reviewers approve
- CI checks pass
- Open concerns resolved or explicitly accepted

## Reviewer Model (Target)
- Required reviewer set (planned): `claude`, `gemini`, `codex`
- Current state: manual/mixed process
- Future: automated orchestrator for multi-agent review requests

## Merge & Notify
After consensus + CI green:
1. Merge PR
2. Post concise notify message to owner with:
   - PR link
   - key changes
   - follow-ups (if any)
