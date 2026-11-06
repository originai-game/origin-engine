#!/usr/bin/env bash
# Validate and land the current branch on origin/main. No force pushes.
set -eu -o pipefail
cd "$(dirname "$0")/.."
. scripts/lock.sh

clean() {
  [ -z "$(git status --porcelain)" ] || {
    echo 'land: commit or remove working-tree changes first' >&2; exit 1;
  }
}

# Conservative shared inputs, including all generated/drift-checked outputs.
# NUL delimiters in the comparison preserve spaces and unusual filenames.
disjoint() {
  python3 - "$1" "$2" "$3" <<'PY'
import subprocess
import sys

base, head, main = sys.argv[1:]
def paths(end):
    return set(subprocess.check_output([
        'git', 'diff', '--no-renames', '--name-only', '-z', base, end
    ]).split(b'\0')) - {b''}

ours, theirs = paths(head), paths(main)
common = (
    b'Cargo.toml', b'Cargo.lock', b'rust-toolchain.toml', b'.cargo',
    b'.config', b'scripts', b'tests/gpu.rs', b'packages/design',
    b'packages/engine/types', b'packages/engine/schema',
    b'packages/knowledge/wiki', b'crates/og-api', b'crates/og-idl',
    b'crates/og-ui/src/theme.rs',
)
shared = any(
    b'/generated/' in p or p.endswith(b'.d.ts') or
    any(p == root or p.startswith(root + b'/') for root in common)
    for p in ours | theirs
)
sys.exit(0 if not shared and ours.isdisjoint(theirs) else 1)
PY
}

clean
git symbolic-ref -q HEAD >/dev/null || { echo 'land: use a branch' >&2; exit 1; }
git fetch origin main
base="$(git rev-parse FETCH_HEAD)"
git rebase "$base"
while :; do
  head="$(git rev-parse HEAD)"
  if [ "$head" = "$base" ]; then echo "land: already on origin/main ($head)"; exit 0; fi
  # Never hold the landing lock while acquiring or running the gate.
  scripts/check.sh quick
  clean
  [ "$(git rev-parse HEAD)" = "$head" ] || { echo 'land: HEAD changed during gate' >&2; exit 1; }
  while :; do
    og_lock_acquire "$(git rev-parse --git-common-dir)/og-landing" "scripts/land.sh ($PWD)"
    git fetch origin main
    main="$(git rev-parse origin/main)"
    if [ "$main" != "$base" ]; then
      git merge-base --is-ancestor "$base" "$main" || {
        echo 'land: origin/main was rewritten; inspect before retrying' >&2; exit 1;
      }
      reuse=0
      disjoint "$base" "$head" "$main" && reuse=1
      git rebase "$main"
      clean
      base="$main"
      head="$(git rev-parse HEAD)"
      if [ "$reuse" = 0 ]; then
        echo 'land: overlapping or shared inputs; rechecking affected quick steps'
        og_lock_release
        break
      fi
      echo 'land: disjoint changes, reusing passing quick gate'
    fi
    if git push origin HEAD:refs/heads/main; then
      echo "land: pushed $head to origin/main"
      exit 0
    fi
    # A writer not using this command can still race us. Verify whether
    # main advanced (or accepted a push whose response was lost).
    git fetch origin main
    if git merge-base --is-ancestor "$head" origin/main; then
      echo "land: origin/main contains $head"; exit 0
    fi
    [ "$(git rev-parse origin/main)" != "$base" ] || {
      echo 'land: push failed without main advancing; inspect remote access' >&2; exit 1;
    }
    og_lock_release
  done
done
