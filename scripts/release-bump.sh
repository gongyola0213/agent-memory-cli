#!/usr/bin/env bash
set -euo pipefail

# Bump version using prefix rules and optionally create tag.
# Usage:
#   scripts/release-bump.sh [--mode auto|patch|minor|major] [--message "feat: ..."] [--tag]

MODE="auto"
MESSAGE="$(git log -1 --pretty=%s)"
DO_TAG="false"

while [ $# -gt 0 ]; do
  case "$1" in
    --mode) MODE="$2"; shift 2 ;;
    --message) MESSAGE="$2"; shift 2 ;;
    --tag) DO_TAG="true"; shift 1 ;;
    *) echo "unknown arg: $1" >&2; exit 1 ;;
  esac
done

NEXT="$(scripts/next-version.sh "$MODE" "$MESSAGE")"
if [ "$NEXT" = "none" ]; then
  echo "No version bump required for message: $MESSAGE"
  exit 0
fi

sed -i "s/^version = \"[0-9]\+\.[0-9]\+\.[0-9]\+\"/version = \"$NEXT\"/" Cargo.toml

if [ ! -f CHANGELOG.md ]; then
  cat > CHANGELOG.md <<'EOC'
# Changelog

All notable changes to this project will be documented in this file.
EOC
fi

DATE="$(date +%Y-%m-%d)"
TMP=$(mktemp)
{
  echo "# Changelog"
  echo
  echo "## v$NEXT - $DATE"
  echo "- Automated version bump from commit/PR prefix rules"
  echo
  tail -n +3 CHANGELOG.md 2>/dev/null || true
} > "$TMP"
mv "$TMP" CHANGELOG.md

git add Cargo.toml CHANGELOG.md
git commit -m "release: v$NEXT"

if [ "$DO_TAG" = "true" ]; then
  git tag "v$NEXT"
  echo "Created tag v$NEXT"
fi

echo "Bumped to v$NEXT"
