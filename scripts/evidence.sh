#!/usr/bin/env bash
# Files a milestone evidence file (tests/evidence/README.md):
#
#   scripts/evidence.sh <milestone> <slug> <file> "<how it was produced>"
#
#   milestone  M0..M7 or E1..E9 (docs/phase-1.md §2)
#   slug       lowercase words joined by '-', e.g. spike-frame-native
#   file       a .png, .json or .md file; its extension is kept
#   note       the exact command or procedure that produced the file
#
# Copies <file> to tests/evidence/<milestone>/<slug>.<ext> and records it in
# tests/evidence/<milestone>/NOTES.md (replacing that file's previous line).
set -eu -o pipefail
# shellcheck source-path=SCRIPTDIR
. "$(dirname "$0")/lib.sh"

usage() { og_err "usage: scripts/evidence.sh <M0..M7 or E1..E9> <slug> <file.png|json|md> \"<how it was produced>\""; exit 2; }
[ "$#" -eq 4 ] || usage
milestone="$1"; slug="$2"; src="$3"; note="$4"

printf '%s' "$milestone" | grep -Eq '^M[0-9]+$' || { og_err "milestone must look like M0, got '$milestone'"; usage; }
printf '%s' "$slug" | grep -Eq '^[a-z0-9]+(-[a-z0-9]+)*$' || { og_err "slug must be lowercase words joined by '-', got '$slug'"; usage; }
[ -f "$src" ] || { og_err "no such file: $src"; exit 1; }
[ -n "$(printf '%s' "$note" | tr -d '[:space:]')" ] || { og_err "the note must say how the file was produced"; usage; }
ext="${src##*.}"
case "$ext" in png|json|md) ;; *) og_err "evidence must be .png, .json or .md, got .$ext"; exit 1 ;; esac
[ "$slug" != notes ] && [ "$slug" != readme ] || { og_err "slug '$slug' is reserved"; exit 1; }

dir="$OG_ROOT/tests/evidence/$milestone"
dest="$dir/$slug.$ext"
notes="$dir/NOTES.md"
mkdir -p "$dir"
cp "$src" "$dest"

if command -v sha256sum >/dev/null; then sum="$(sha256sum "$dest" | awk '{print $1}')"
else sum="$(shasum -a 256 "$dest" | awk '{print $1}')"; fi
rev="$(git -C "$OG_ROOT" rev-parse --short HEAD 2>/dev/null || echo no-git)"
if ! git -C "$OG_ROOT" diff --quiet HEAD -- . ':!tests/evidence' 2>/dev/null; then rev="$rev+dirty"; fi
day="$(date -u +%Y-%m-%d)"
note="$(printf '%s' "$note" | tr '\n' ' ')"

if [ ! -f "$notes" ]; then
  printf '# %s evidence\n\nOne line per file: what it is, how it was produced, at which revision.\n\n' "$milestone" > "$notes"
fi
tmp="$(mktemp)"
grep -v -F -- "- \`$slug.$ext\` " "$notes" > "$tmp" || true
# shellcheck disable=SC2016 # literal backticks
printf -- '- `%s.%s` — %s (rev %s, %s, sha256 %s)\n' "$slug" "$ext" "$note" "$rev" "$day" "${sum:0:16}" >> "$tmp"
mv "$tmp" "$notes"
og_say "filed tests/evidence/$milestone/$slug.$ext"
tail -n 1 "$notes"
