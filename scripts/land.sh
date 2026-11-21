#!/usr/bin/env bash
# Validate and land the current branch on origin/main. No force pushes.
set -eu -o pipefail
cd "$(dirname "$0")/.."
. scripts/lock.sh

clean() {
  [ -z "$(git status --porcelain)" ] || {
    echo 'land: commit or remove working-tree changes first' >&2; exit 1;
  }
  # Pinned assets are ignored by Git: a re-recorded reference shows only here.
  bun scripts/assets.ts status >/dev/null || {
    echo 'land: pin the changed assets (bun scripts/assets.ts pin) and push them, or restore them (ensure)' >&2; exit 1;
  }
}

# Shared inputs: everything a gate step reads. Only prose (docs, tasks,
# changelog fragments, README, AGENTS) reads nothing the quick steps read, so
# only prose-only deltas reuse a passing gate without a recheck; a code delta
# on either side rechecks the affected steps (2026-11-21: five red steps a
# cold run fails landed through the old wider exception). NUL delimiters in
# the comparison preserve spaces and unusual filenames.
disjoint() {
  python3 - "$1" "$2" "$3" <<'PY'
import subprocess
import sys

base, head, main = sys.argv[1:]
def paths(end):
    return set(subprocess.check_output([
        'git', 'diff', '--no-renames', '--name-only', '-z', base, end
    ]).split(b'\0')) - {b''}

def prose(p):
    if p in (b'AGENTS.md', b'README.md'):
        return True
    parts = p.split(b'/')
    if parts[0] in (b'docs', b'tasks'):
        return p.endswith(b'.md')
    return parts[:2] == [b'changelog', b'unreleased']

ours, theirs = paths(head), paths(main)
sys.exit(0 if all(prose(p) for p in ours | theirs) and ours.isdisjoint(theirs) else 1)
PY
}

# The recheck holds the landing queue: scripts/check.sh gives it a lone
# gate's cores, every other gate a lower priority, and a measurement yields
# to it. Per-step and per-test-binary input records run only what the
# rebase changed since this checkout's last pass, in its own target.
recheck() {
  git checkout -q --detach "$1"
  OG_GATE_LANDING=1 OG_LANDING_RECHECK=1 scripts/check.sh quick && clean &&
    { [ "$(git rev-parse HEAD)" = "$1" ] || { echo 'land: HEAD changed during gate' >&2; false; }; }
}

# Waiting landers whose first quick passed, oldest first, as "offer head
# worktree", leaving out those this drain already settled. A dead lander's
# offer, or one whose branch moved since, is dropped.
offers() {
  for offer in "$queue/batch/"*; do
    [ -f "$offer" ] || continue
    case " $settled " in *" $offer "*) continue ;; esac
    read -r their tree < "$offer" || continue
    if kill -0 "${offer##*.}" 2>/dev/null && [ "$(git -C "$tree" rev-parse HEAD 2>/dev/null)" = "$their" ]; then
      echo "$offer $their $tree"
    else
      rm -f "$offer"
    fi
  done
}

# Stack every offer on tips[0], each rebased onto the one before
# (generated-only conflicts regenerated). tips[k] is then tips[0] with the
# first k stacked; theirs and files name each stacked lander's head and
# offer. An offer that does not rebase cleanly is settled: it lands alone.
stack() {
  theirs=() files=()
  while read -r offer their tree <&3; do
    [ -n "$offer" ] || continue
    git checkout -q --detach "$their"
    if python3 scripts/rebase-generated.py "${tips[${#theirs[@]}]}" </dev/null && [ -z "$(git status --porcelain)" ]; then
      theirs+=("$their") files+=("$offer") tips+=("$(git rev-parse HEAD)")
      echo "land: batching $their ($tree)"
    else
      git rebase --abort 2>/dev/null || true
      git reset -q --hard
      settled="$settled $offer"
      echo "land: $their does not stack on main; it lands on its own"
    fi
  done 3<<OFFERS
$(offers)
OFFERS
}

# Find the longest prefix of tips that passes quick, above tips[$1] (already
# known to pass; -1 when none is). The whole stack usually passes in one
# recheck; otherwise bisection finds the first head failing on those before
# it, which goes back to its owner (or fails this lander). Sets head and
# landing to what the push lands; false when nothing new passed.
prefix() {
  local known="$1" lo="$1" hi=$((${#tips[@]} - 1)) mid k
  [ "${#theirs[@]}" = 0 ] || echo "land: rechecking ${#theirs[@]} waiting landing(s) as one stack"
  if recheck "${tips[$hi]}"; then
    lo="$hi"
  else
    while [ $((hi - lo)) -gt 1 ]; do
      mid=$(((lo + hi) / 2))
      echo "land: the stack failed quick; rechecking its first $mid"
      if recheck "${tips[$mid]}"; then lo="$mid"; else hi="$mid"; fi
    done
    if [ "$hi" = 0 ]; then
      own=1
      echo 'land: this branch fails quick on origin/main; draining the queue without it' >&2
    else
      k=$((hi - 1))
      printf '%s %s\n' "${tips[$k]}" "$PWD/$(ls -td target/evidence/gate/runs/* | head -n 1)" > "$queue/failed/${theirs[$k]}"
      rm -f "${files[$k]}"
      settled="$settled ${files[$k]}"
      echo "land: ${theirs[$k]} fails quick on the stack; handed back to its owner"
    fi
  fi
  head="${tips[$lo]}" landing=()
  for ((k = 0; k < lo; k++)); do landing+=("$k"); done
  [ "$lo" -gt "$known" ]
}

# Rebase what the push carries, tips[0..n] for the n landers in it, onto an
# origin/main moved by a writer outside this queue (another orb's land.sh).
# Each tip becomes the rebased copy of its last commit that survived (a
# commit already on main is dropped); was keeps the old head. False on a
# conflict that is not generated-only.
retip() {
  local n="${#landing[@]}"
  was="$head"
  tips=("${tips[@]:0:n+1}")
  theirs=(${theirs[@]+"${theirs[@]:0:n}"}) files=(${files[@]+"${files[@]:0:n}"})
  git checkout -q --detach "$head"
  if ! python3 scripts/rebase-generated.py origin/main </dev/null || [ -n "$(git status --porcelain)" ]; then
    git rebase --abort 2>/dev/null || true
    git reset -q --hard
    return 1
  fi
  head="$(git rev-parse HEAD)"
  mapfile -t tips < <(python3 - "$was" "$head" "${tips[@]}" <<'PY'
import subprocess
import sys

was, head, *tips = sys.argv[1:]
def commits(end):
    out = subprocess.check_output(['git', 'log', '--reverse', '--format=%H%x00%s', 'origin/main..' + end])
    return [line.split('\0', 1) for line in out.decode().splitlines()]
main = subprocess.check_output(['git', 'rev-parse', 'origin/main']).decode().strip()
new, last, j, at = commits(head), main, 0, {}
for commit, subject in commits(was):
    if j < len(new) and new[j][1] == subject:
        last, j = new[j][0], j + 1
    at[commit] = last
print(*[at.get(t, main) for t in tips[:-1]], head, sep='\n')
PY
)
  [ "${#tips[@]}" = $((n + 1)) ] || return 1
  # This lander's own branch follows its rebased head, as after the first rebase.
  [ "$known" = 0 ] || git update-ref "refs/heads/$branch" "${tips[0]}"
}

# Push head; then tell each stacked lander in landing its commits landed.
# When a writer outside this queue moved origin/main first, rebase onto it
# and push again: without a gate when what landed is disjoint from the
# pushed change, else after rechecking the steps the rebase affected. Stops
# when that recheck fails (prefix lands any passing part of a stack).
push() {
  local old remote
  remote="$(git rev-parse origin/main)"
  until git push origin "$head:refs/heads/main"; do
    # The push response may be lost; retry only when another writer moved main.
    git fetch origin main
    if git merge-base --is-ancestor "$head" origin/main; then
      echo "land: origin/main contains $head"
      break
    fi
    if [ "$(git rev-parse origin/main)" = "$remote" ]; then
      echo 'land: push failed and origin/main did not move; fix the push failure and run scripts/land.sh again' >&2
      finish 1
    fi
    remote="$(git rev-parse origin/main)"
    old="$(git merge-base "$head" origin/main)"
    echo "land: origin/main moved to $(git rev-parse --short origin/main) outside this queue; rebasing onto it"
    retip || {
      echo 'land: the head conflicts with the new origin/main; rebase it and run scripts/land.sh again' >&2
      finish 1
    }
    if disjoint "$old" "$was" origin/main; then
      echo 'land: disjoint from what landed; pushing without a recheck'
    else
      echo 'land: overlapping or shared inputs; rechecking affected quick steps'
      prefix "$known" || return 0
    fi
  done
  echo "land: pushed $head to origin/main"
  for k in "${landing[@]+"${landing[@]}"}"; do
    printf '%s\n' "${tips[$((k + 1))]}" > "$queue/landed/${theirs[$k]}"
    rm -f "${files[$k]}"
    settled="$settled ${files[$k]}"
  done
}

# Exit when the lander holding the queue settled this head: landed (the
# branch moves to its landed commit) or handed back after failing quick.
mine() {
  if [ -f "$queue/failed/$head" ]; then
    read -r on logs < "$queue/failed/$head"
    rm -f "$queue/failed/$head" "$offer"
    echo "land: stacked on $on by the lander ahead, this head failed quick (logs: $logs)" >&2
    echo 'land: the heads before it landed; fix this branch and run scripts/land.sh again' >&2
    exit 1
  fi
  if [ -f "$queue/landed/$head" ]; then
    tip="$(cat "$queue/landed/$head")"
    rm -f "$queue/landed/$head" "$offer"
    if [ "$(git rev-parse HEAD)" = "$head" ]; then git reset -q --keep "$tip"; fi
    echo "land: landed in a batch as $tip"
    exit 0
  fi
}

finish() {
  git symbolic-ref -q HEAD >/dev/null || git checkout -q "$branch"
  exit "$1"
}

# Parse the whole function before rebasing: replay may replace this file.
# A merge queue: the first gate runs outside the queue, so landers' gates
# overlap. A lander that passed offers its head and waits. One FIFO landing
# lock is held through the pushes, so main moves only under it; its holder
# lands itself, then drains the queue: everyone waiting, rebased as one
# stack, one recheck, one push, again until no offer is left.
land() {
clean
og_temp_prepare "$PWD"
git symbolic-ref -q HEAD >/dev/null || { echo 'land: use a branch' >&2; exit 1; }
branch="$(git symbolic-ref --short HEAD)"
git fetch origin main
base="$(git rev-parse FETCH_HEAD)"
python3 scripts/rebase-generated.py "$base"
head="$(git rev-parse HEAD)"
if [ "$head" = "$base" ]; then echo "land: already on origin/main ($head)"; exit 0; fi
# A tree that passed the whole full lane with --all (scripts/gate/run.py's
# lane.full record) lands without the opening quick; a later recheck still
# runs what a rebase onto a moved main changed.
# Where scripts/lib.sh puts the target directory (land.sh sources only lock.sh).
if [ "$(cat "${CARGO_TARGET_DIR:-target}/og-gate/green/lane.full" 2>/dev/null)" = "$(git rev-parse "HEAD^{tree}")" ]; then
  echo "land: $head passed full --all on this tree; skipping the opening quick"
else
  OG_GATE_LANDING=1 scripts/check.sh quick
fi
clean
[ "$(git rev-parse HEAD)" = "$head" ] || { echo 'land: HEAD changed during gate' >&2; exit 1; }

queue="$(git rev-parse --git-common-dir)/og-landing"
mkdir -p "$queue/batch" "$queue/landed" "$queue/failed"
offer="$queue/batch/$(date +%s).$$"
printf '%s %s\n' "$head" "$PWD" > "$offer"
# Wait while a live holder drains the queue; it may land or hand back this
# head meanwhile, and mine then settles it.
while :; do
  mine
  og_owner_alive "$queue/og-gate.lock" || break
  sleep 2
done
og_lock_acquire "$queue" "scripts/land.sh ($PWD)"
rm -f "$offer"
mine
settled="" own=0 tips=() theirs=() files=() landing=()
git fetch origin main
main="$(git rev-parse origin/main)"
recheck=0
if [ "$main" != "$base" ]; then
  git merge-base --is-ancestor "$base" "$main" || {
    echo 'land: origin/main was rewritten; inspect before retrying' >&2; exit 1;
  }
  disjoint "$base" "$head" "$main" || recheck=1
  python3 scripts/rebase-generated.py "$main"
  clean
  head="$(git rev-parse HEAD)"
  if [ "$recheck" = 0 ]; then echo 'land: disjoint changes, reusing passing quick gate'; fi
fi
tips=("$head") known=-1
if [ "$recheck" = 1 ]; then
  echo 'land: overlapping or shared inputs; rechecking affected quick steps at the head of the queue'
  stack
  if prefix -1; then push; fi
else
  push
fi
# Drain: whoever waits now is rebased onto main as one stack, rechecked once
# and pushed together, until the queue is empty.
while :; do
  git fetch -q origin main
  [ -n "$(offers)" ] || break
  tips=("$(git rev-parse origin/main)") known=0
  git checkout -q --detach "${tips[0]}"
  stack
  [ "${#theirs[@]}" -gt 0 ] || continue
  if prefix 0; then push; fi
done
finish "$own"
}
land
