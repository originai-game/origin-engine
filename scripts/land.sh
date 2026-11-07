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

# Parse the whole function before rebasing: replay may replace this file.
# A merge queue: the first gate runs outside the queue, so landers' gates
# overlap; then one FIFO landing lock is held through the push, so main
# moves only under it and a lander rechecks at most once.
land() {
clean
og_temp_prepare "$PWD"
git symbolic-ref -q HEAD >/dev/null || { echo 'land: use a branch' >&2; exit 1; }
git fetch origin main
base="$(git rev-parse FETCH_HEAD)"
python3 scripts/rebase-generated.py "$base"
head="$(git rev-parse HEAD)"
if [ "$head" = "$base" ]; then echo "land: already on origin/main ($head)"; exit 0; fi
scripts/check.sh quick
clean
[ "$(git rev-parse HEAD)" = "$head" ] || { echo 'land: HEAD changed during gate' >&2; exit 1; }

og_lock_acquire "$(git rev-parse --git-common-dir)/og-landing" "scripts/land.sh ($PWD)"
git fetch origin main
main="$(git rev-parse origin/main)"
if [ "$main" != "$base" ]; then
  git merge-base --is-ancestor "$base" "$main" || {
    echo 'land: origin/main was rewritten; inspect before retrying' >&2; exit 1;
  }
  reuse=0
  disjoint "$base" "$head" "$main" && reuse=1
  python3 scripts/rebase-generated.py "$main"
  clean
  head="$(git rev-parse HEAD)"
  if [ "$reuse" = 1 ]; then
    echo 'land: disjoint changes, reusing passing quick gate'
  else
    echo 'land: overlapping or shared inputs; rechecking affected quick steps at the head of the queue'
    scripts/check.sh quick
    clean
    [ "$(git rev-parse HEAD)" = "$head" ] || { echo 'land: HEAD changed during gate' >&2; exit 1; }
  fi
fi
if git push origin HEAD:refs/heads/main; then
  echo "land: pushed $head to origin/main"
  exit 0
fi
# The push response may be lost; anything else is a writer outside the queue.
git fetch origin main
if git merge-base --is-ancestor "$head" origin/main; then
  echo "land: origin/main contains $head"; exit 0
fi
echo 'land: origin/main moved outside scripts/land.sh; run it again' >&2
exit 1
}
land
