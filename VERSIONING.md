# Versioning & Release Policy

## Prefix rules (source of truth)
- `fix:` -> patch bump (`v0.1.1`)
- `feat:` -> minor bump (`v0.2.0`)
- `breaking:` or `!` marker (e.g. `feat!:`) -> major bump (`v1.0.0`)
- `docs/chore/test/refactor` -> no automatic bump

Priority when mixed: `major > minor > patch`.

## Automation flow
1. PR merges into `main` with conventional prefix in title/commit subject.
2. `Version Bump` workflow runs on `main` push.
3. If bump is needed, workflow:
   - updates `Cargo.toml` version
   - updates `CHANGELOG.md`
   - commits `release: vX.Y.Z`
   - pushes tag `vX.Y.Z`
4. `Release` workflow (tag-triggered) builds Linux/macOS/Windows binaries and publishes artifacts.

## Safety
- Workflow skips bump on commits already starting with `release: v` to avoid loops.
- For manual override, use `scripts/release-bump.sh --mode patch|minor|major`.
