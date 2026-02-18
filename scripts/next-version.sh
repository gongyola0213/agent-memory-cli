#!/usr/bin/env bash
set -euo pipefail

# Usage:
#   scripts/next-version.sh [patch|minor|major|auto] [message]
# Defaults:
#   mode=auto, message=last git commit subject

MODE="${1:-auto}"
MSG="${2:-$(git log -1 --pretty=%s)}"

current_version() {
  sed -n 's/^version = "\([0-9]\+\.[0-9]\+\.[0-9]\+\)"/\1/p' Cargo.toml | head -n1
}

bump() {
  local v="$1" kind="$2"
  IFS='.' read -r major minor patch <<< "$v"
  case "$kind" in
    patch) patch=$((patch+1)) ;;
    minor) minor=$((minor+1)); patch=0 ;;
    major) major=$((major+1)); minor=0; patch=0 ;;
    *) echo "invalid bump kind: $kind" >&2; exit 1 ;;
  esac
  echo "${major}.${minor}.${patch}"
}

kind_from_message() {
  local m="$1"
  # Priority: major > minor > patch
  if echo "$m" | grep -Eq '^(breaking:|.*!:)' ; then
    echo "major"; return
  fi
  if echo "$m" | grep -Eq '^feat(\(.+\))?:' ; then
    echo "minor"; return
  fi
  if echo "$m" | grep -Eq '^fix(\(.+\))?:' ; then
    echo "patch"; return
  fi
  echo "none"
}

CUR="$(current_version)"
if [ -z "$CUR" ]; then
  echo "could not read current version from Cargo.toml" >&2
  exit 1
fi

if [ "$MODE" = "auto" ]; then
  KIND="$(kind_from_message "$MSG")"
else
  KIND="$MODE"
fi

if [ "$KIND" = "none" ]; then
  echo "none"
  exit 0
fi

NEXT="$(bump "$CUR" "$KIND")"
echo "$NEXT"
