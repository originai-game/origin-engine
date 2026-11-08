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

# The recheck is the head of the queue: scripts/check.sh gives it the cores
# of a lone gate and lowers every other gate's priority.
recheck() {
  OG_LANDING_RECHECK=1 scripts/check.sh quick && clean &&
    { [ "$(git rev-parse HEAD)" = "$1" ] || { echo 'land: HEAD changed during gate' >&2; false; }; }
}

# Batch: stack every waiting lander whose quick already passed, oldest first,
# on this rebased head (detached), so one recheck covers the whole stack and
# one push lands it. A branch that does not rebase cleanly (generated-only
# conflicts are regenerated) is left out to land on its own. Sets stack and
# landed ("old-head:new-tip" for each stacked lander).
batch() {
  stack="$head"
  for offer in "$queue/batch/"*; do
    [ -f "$offer" ] || continue
    read -r their tree < "$offer" || continue
    if ! kill -0 "${offer##*.}" 2>/dev/null || [ "$(git -C "$tree" rev-parse HEAD 2>/dev/null)" != "$their" ]; then
      rm -f "$offer"; continue
    fi
    git checkout -q --detach "$their"
    if python3 scripts/rebase-generated.py "$stack" && [ -z "$(git status --porcelain)" ]; then
      stack="$(git rev-parse HEAD)"
      landed="$landed $their:$stack"
      echo "land: batching $their ($tree)"
    else
      git rebase --abort 2>/dev/null || true
      git reset -q --hard
      echo "land: $their does not stack here; it lands on its own"
    fi
  done
  git checkout -q --detach "$stack"
}

# After the push: tell each stacked lander its commits landed, and return
# this checkout to its branch.
pushed() {
  for pair in $landed; do
    printf '%s\n' "${pair#*:}" > "$queue/landed/${pair%%:*}"
  done
  if [ -n "$landed" ]; then git checkout -q "$branch"; fi
  exit 0
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

queue="$(git rev-parse --git-common-dir)/og-landing"
mkdir -p "$queue/batch" "$queue/landed"
# Offer this passing head to the lander ahead in the queue (see batch).
offer="$queue/batch/$(date +%s).$$"
printf '%s %s\n' "$head" "$PWD" > "$offer"
og_lock_acquire "$queue" "scripts/land.sh ($PWD)"
rm -f "$offer"
if [ -f "$queue/landed/$head" ]; then
  tip="$(cat "$queue/landed/$head")"
  rm -f "$queue/landed/$head"
  git fetch origin main
  if git merge-base --is-ancestor "$tip" origin/main; then
    if [ "$(git rev-parse HEAD)" = "$head" ]; then git reset -q --keep "$tip"; fi
    echo "land: landed in a batch as $tip"
    exit 0
  fi
fi
git fetch origin main
main="$(git rev-parse origin/main)"
landed=""
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
    branch="$(git symbolic-ref --short HEAD)"
    batch
    if [ -n "$landed" ] && recheck "$stack"; then
      head="$stack"
    else
      if [ -n "$landed" ]; then echo 'land: the batch failed quick; rechecking this branch alone'; fi
      landed=""
      git checkout -q "$branch"
      echo 'land: overlapping or shared inputs; rechecking affected quick steps at the head of the queue'
      recheck "$head" || exit 1
    fi
  fi
fi
if git push origin HEAD:refs/heads/main; then
  echo "land: pushed $head to origin/main"
  pushed
fi
# The push response may be lost; anything else is a writer outside the queue.
git fetch origin main
if git merge-base --is-ancestor "$head" origin/main; then
  echo "land: origin/main contains $head"; pushed
fi
echo 'land: origin/main moved outside scripts/land.sh; run it again' >&2
exit 1
}
land
