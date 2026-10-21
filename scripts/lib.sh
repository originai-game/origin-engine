# shellcheck shell=bash disable=SC2034 # variables are used by the sourcing scripts
# shellcheck source-path=SCRIPTDIR
# Shared helpers for scripts/*.sh. Sourced, never executed. Bash 3.2 compatible
# (the macOS system bash), so no associative arrays or mapfile.

OG_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
OG_TARGET_DIR="${CARGO_TARGET_DIR:-$OG_ROOT/target}"
case "$OG_TARGET_DIR" in /*) ;; *) OG_TARGET_DIR="$OG_ROOT/$OG_TARGET_DIR" ;; esac
# Test harnesses (gfx diffs, perf reports) write here; CI uploads it on failure.
export OG_EVIDENCE_DIR="${OG_EVIDENCE_DIR:-$OG_TARGET_DIR/evidence}"
export OG_WASM_OUT="$OG_TARGET_DIR/og-wasm"
# shellcheck source=lock.sh
. "$OG_ROOT/scripts/lock.sh"

og_say() { printf '%s\n' "$*"; }
og_err() { printf '%s\n' "$*" >&2; }

# og_conf FILE KEY: the second field of the first line whose first field is KEY.
og_conf() {
  awk -v k="$2" '{ sub(/#.*/, "") } $1 == k { print $2; found = 1; exit } END { exit !found }' "$1"
}

# og_budget KEY: a value from scripts/budgets.txt; fails loudly when missing.
og_budget() {
  local v
  if ! v="$(og_conf "$OG_ROOT/scripts/budgets.txt" "$1")" || [ -z "$v" ]; then
    og_err "scripts/budgets.txt has no '$1' entry"
    return 1
  fi
  printf '%s\n' "$v"
}

# og_wasm_crates STATUS: crate names from scripts/wasm-crates.txt with that status.
og_wasm_crates() {
  awk -v s="$1" '{ sub(/#.*/, "") } NF >= 2 && $2 == s { print $1 }' "$OG_ROOT/scripts/wasm-crates.txt"
}

# og_wasm_publish_crates: crates whose line in scripts/wasm-crates.txt says `publish`.
og_wasm_publish_crates() {
  awk '{ sub(/#.*/, "") } NF >= 3 && $3 == "publish" { print $1 }' "$OG_ROOT/scripts/wasm-crates.txt"
}

# og_wasm_lanes_crates: publish crates whose line also says `lanes` (the
# atomics variant scripts/wasm-lanes.sh builds).
og_wasm_lanes_crates() {
  awk '{ sub(/#.*/, "") } NF >= 4 && $3 == "publish" && $4 == "lanes" { print $1 }' "$OG_ROOT/scripts/wasm-crates.txt"
}

# og_wasm_variants CRATE: "NAME FEATURES" for each `variant` line of
# scripts/wasm-crates.txt naming CRATE (the publish variants per game
# profile); nothing when the crate has none (one publish variant, as before).
og_wasm_variants() {
  awk -v c="$1" '{ sub(/#.*/, "") } NF >= 4 && $1 == "variant" && $3 == c { print $2, $4 }' "$OG_ROOT/scripts/wasm-crates.txt"
}

# og_wasm_lanes_variants CRATE: "NAME FEATURES" for each `lanes` line of
# scripts/wasm-crates.txt naming CRATE (the lanes modules per game profile,
# in file order: the 2D one first, so the 3D one is cargo's last output in
# the shared lanes target directory); nothing when the crate has none (one
# lanes variant with the publish feature, both profiles).
og_wasm_lanes_variants() {
  awk -v c="$1" '{ sub(/#.*/, "") } NF >= 4 && $1 == "lanes" && $3 == c { print $2, $4 }' "$OG_ROOT/scripts/wasm-crates.txt"
}

# og_lock_version PACKAGE: the version Cargo.lock pins for a crates.io package.
og_lock_version() {
  awk -v p="$1" '
    $0 == "[[package]]" { name = "" }
    $1 == "name" && $3 == "\"" p "\"" { name = p }
    $1 == "version" && name == p { gsub(/"/, "", $3); print $3; exit }
  ' "$OG_ROOT/Cargo.lock"
}

og_need() {
  if ! command -v "$1" >/dev/null 2>&1; then
    og_err "missing tool: $1 (run scripts/install-tools.sh)"
    return 1
  fi
}

# og_words: stdin words joined by single spaces.
og_words() { tr -s ' \n\t' '   ' | sed 's/^ //; s/ $//'; }

og_bytes() { wc -c < "$1" | tr -d ' '; }

# og_sha256 FILE: the file's SHA-256, hex.
og_sha256() {
  if command -v sha256sum >/dev/null 2>&1; then sha256sum "$1" | awk '{print $1}'
  else shasum -a 256 "$1" | awk '{print $1}'; fi
}

# Results keyed by content, kept across gate runs (wasm-opt and size-gate
# reuse them for a byte-identical input).
OG_CONTENT_CACHE="$OG_TARGET_DIR/og-wasm-cache"

# og_gate_excluded: crate names from scripts/gate-exclude.txt.
og_gate_excluded() {
  awk '{ sub(/#.*/, "") } NF >= 1 { print $1 }' "$OG_ROOT/scripts/gate-exclude.txt"
}

# og_workspace_crates: names of the workspace's own packages.
og_workspace_crates() {
  cargo metadata --no-deps --format-version 1 \
    | grep -o '"name":"[^"]*","version":"[^"]*","id":"path+file' \
    | sed 's/^"name":"\([^"]*\)".*/\1/'
}

# --- Path-aware steps (scripts/check.sh, docs/ci.md §Lanes) -----------------
# Each step that passes records the tree it passed on in
# <target>/og-gate/green/<step>: a git tree of the whole working tree,
# untracked files included (a private index, so the real one is untouched).
# A later run compares that tree with this one (git diff --name-only) and
# skips a step none of whose inputs changed.
OG_GATE_STATE="$OG_TARGET_DIR/og-gate"

# og_worktree_tree: the tree hash of the working tree as it is now.
og_worktree_tree() {
  local idx="$OG_GATE_STATE/index" real
  mkdir -p "$OG_GATE_STATE" || return 1
  real="$(git -C "$OG_ROOT" rev-parse --git-path index 2>/dev/null)" || return 1
  case "$real" in /*) ;; *) real="$OG_ROOT/$real" ;; esac
  # Starting from the real index makes git re-hash only what changed.
  cp "$real" "$idx" 2>/dev/null || rm -f "$idx"
  GIT_INDEX_FILE="$idx" git -C "$OG_ROOT" add -A . >/dev/null 2>&1 || return 1
  GIT_INDEX_FILE="$idx" git -C "$OG_ROOT" write-tree 2>/dev/null
}

og_green_get() { cat "$OG_GATE_STATE/green/$1" 2>/dev/null; }
og_green_set() {
  [ -n "${OG_GATE_TREE:-}" ] || return 0
  mkdir -p "$OG_GATE_STATE/green" && printf '%s\n' "$OG_GATE_TREE" > "$OG_GATE_STATE/green/$1"
}
og_green_clear() { rm -f "$OG_GATE_STATE/green/$1"; }

# og_unchanged KEY PATH...: true when KEY last passed on a tree that differs
# from OG_GATE_TREE in no PATH (a file, or a directory and all under it).
# Sets OG_GREEN to that tree. False when anything is unknown.
og_unchanged() {
  local key="$1" green changed f p
  shift
  OG_GREEN=""
  [ -n "${OG_GATE_TREE:-}" ] || return 1
  green="$(og_green_get "$key")" && [ -n "$green" ] || return 1
  changed="$(git -C "$OG_ROOT" diff --no-renames --name-only "$green" "$OG_GATE_TREE" 2>/dev/null)" || return 1
  while IFS= read -r f; do
    [ -n "$f" ] || continue
    for p in "$@"; do
      case "$f" in "$p" | "$p"/*) return 1 ;; esac
    done
  done <<< "$changed"
  OG_GREEN="$green"
  return 0
}

# og_crate_dirs CARGO-TREE-ARGS...: the repository-relative directories of
# the workspace crates in a dependency closure (`cargo tree`); "crates" when
# cargo cannot say, so a rule never misses a crate.
og_crate_dirs() {
  local out
  out="$(cargo tree "$@" --prefix none 2>/dev/null | awk -v root="$OG_ROOT/" '
    { i = index($0, "(" root); if (i) { s = substr($0, i + 1); print substr(s, 1, index(s, ")") - 1) } }
  ' | sort -u | sed "s|^$OG_ROOT/||")"
  if [ -n "$out" ]; then printf '%s\n' "$out"; else echo crates; fi
}
