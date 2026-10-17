#!/usr/bin/env bash
# Regenerates crates/og-ui/src/theme.rs from the platform's design tokens.
#
#   crates/og-ui/scripts/gen-theme.sh            sync tokens.json from the platform, regenerate
#   crates/og-ui/scripts/gen-theme.sh --offline  regenerate from the vendored copy only
#   crates/og-ui/scripts/gen-theme.sh --check    fail when the vendored copy or theme.rs is stale
#
# The platform file is $OG_TOKENS, or packages/tokens/tokens.json of a platform
# checkout next to this repository (../origingame). The engine never reads the
# platform at build time: the copy in crates/og-ui/tokens/ is the pinned input,
# like the gateway mirror's tokens:sync, and tests/theme.rs proves theme.rs is
# exactly what the generator writes from it.
set -eu -o pipefail
here="$(cd "$(dirname "$0")" && pwd)"
crate="$(dirname "$here")"
root="$(cd "$crate/../.." && pwd)"
vendored="$crate/tokens/tokens.json"
theme="$crate/src/theme.rs"
platform="${OG_TOKENS:-$root/../origingame/packages/tokens/tokens.json}"
mode="${1:-sync}"

case "$mode" in
  sync|--check)
    if [ ! -f "$platform" ]; then
      echo "gen-theme: no platform tokens at $platform; set OG_TOKENS=<origingame>/packages/tokens/tokens.json or pass --offline" >&2
      exit 1
    fi ;;
  --offline) ;;
  *) sed -n '2,13p' "$0" | sed 's/^# \{0,1\}//'; exit 2 ;;
esac

if [ "$mode" = --check ]; then
  if ! cmp -s "$platform" "$vendored"; then
    echo "gen-theme: crates/og-ui/tokens/tokens.json differs from $platform; run crates/og-ui/scripts/gen-theme.sh" >&2
    exit 1
  fi
  tmp="$(mktemp)"; trap 'rm -f "$tmp"' EXIT
  (cd "$root" && cargo run -q -p og-ui --example gen-theme -- "$vendored" "$tmp")
  if ! cmp -s "$tmp" "$theme"; then
    echo "gen-theme: crates/og-ui/src/theme.rs is stale or hand-edited; run crates/og-ui/scripts/gen-theme.sh" >&2
    exit 1
  fi
  echo "gen-theme: tokens and theme.rs are current"
  exit 0
fi

if [ "$mode" = sync ]; then
  cp "$platform" "$vendored"
  echo "gen-theme: synced $vendored from $platform"
fi
(cd "$root" && cargo run -q -p og-ui --example gen-theme -- "$vendored" "$theme")
