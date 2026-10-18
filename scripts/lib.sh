# shellcheck shell=bash disable=SC2034 # variables are used by the sourcing scripts
# Shared helpers for scripts/*.sh. Sourced, never executed. Bash 3.2 compatible
# (the macOS system bash), so no associative arrays or mapfile.

OG_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
OG_TARGET_DIR="${CARGO_TARGET_DIR:-$OG_ROOT/target}"
case "$OG_TARGET_DIR" in /*) ;; *) OG_TARGET_DIR="$OG_ROOT/$OG_TARGET_DIR" ;; esac
# Test harnesses (gfx diffs, perf reports) write here; CI uploads it on failure.
export OG_EVIDENCE_DIR="${OG_EVIDENCE_DIR:-$OG_TARGET_DIR/evidence}"
export OG_WASM_OUT="$OG_TARGET_DIR/og-wasm"

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
