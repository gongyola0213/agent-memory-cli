# Versioning & Release Policy

## Versioning
- `fix` -> patch bump (`v0.1.1`)
- `feat` -> minor bump (`v0.2.0`)
- breaking change -> major bump (`v1.0.0`)

## Branch & PR policy
- No direct feature push to `main`
- Use branch -> PR -> CI pass -> merge

## Release policy
- Releases are tag-driven (`v*`)
- Tag push triggers release workflow:
  - Build Linux/macOS/Windows binaries
  - Upload archives + SHA256SUMS
  - Generate release notes
