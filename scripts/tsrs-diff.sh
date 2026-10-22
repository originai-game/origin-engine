#!/usr/bin/env bash
# check-conformance: tsrs (og check's checker) held to tsgo, the TypeScript
# compiler it ports (typescript@TSGO_VERSION, the constant in
# crates/og-cli/src/check.rs), on the same configs with the same checker
# count. A wave's end runs it (scripts/wave-end.sh, stage conformance), and
# a tsrs version move runs it before it lands (docs/ci.md §Toolchain).
#
#   scripts/tsrs-diff.sh [CONFIG...]
#
# Without arguments: packages/engine's three tsconfigs, every template (with
# the paths og check gives a project without node_modules), the five
# examples and the error corpus scripts/tsrs-corpus/ (22 files, one error a
# game author makes each). Each config runs once per checker (`-p CONFIG
# --noEmit --incremental false --pretty false --checkers 4`; tsrs adds
# `--checkerAssignment go --noLazyMembers`, the order closest to tsgo's) and
# each side's diagnostics become (file, line, col, code) keys:
#
#   - one tsgo reports and tsrs does not (a miss) fails, each listed;
#   - one tsrs reports and tsgo does not, or the same key with another
#     message, is printed (`diff` lines) and passes; re-check those with
#     the typescript 7.1.0-dev tsrs ports before calling them tsrs defects;
#   - both clean is one PASS line.
#
# It also holds the pins to their single statements: no template or example
# declares a typescript devDependency (og check is their checker), and
# packages/engine's optionalDependencies @tsrs/* equal TSRS_VERSION of
# scripts/install-tools.sh.
#
# TSRS and TSGO name the binaries (default: tsrs on PATH, `bun x --package
# typescript@TSGO_VERSION tsc`).
set -u -o pipefail
# shellcheck source-path=SCRIPTDIR
. "$(dirname "$0")/lib.sh"
cd "$OG_ROOT" || exit 1

TSGO_VERSION="$(sed -n 's/^pub const TSGO_VERSION: &str = "\(.*\)";/\1/p' crates/og-cli/src/check.rs)"
# shellcheck disable=SC2016 # the $ is sed's, matched literally
TSRS_VERSION="$(sed -n 's/^TSRS_VERSION="${TSRS_VERSION:-\(.*\)}"$/\1/p' scripts/install-tools.sh)"
[ -n "$TSGO_VERSION" ] || { og_err "check-conformance: no TSGO_VERSION in crates/og-cli/src/check.rs"; exit 1; }
[ -n "$TSRS_VERSION" ] || { og_err "check-conformance: no TSRS_VERSION in scripts/install-tools.sh"; exit 1; }
TSRS="${TSRS:-tsrs}"
command -v "$TSRS" >/dev/null || { og_err "check-conformance: no tsrs; run scripts/install-tools.sh"; exit 1; }
if [ -n "${TSGO:-}" ]; then tsgo=("$TSGO"); against="$("$TSGO" --version)"
else og_need bun || exit 1; tsgo=(bun x --package "typescript@$TSGO_VERSION" tsc); against="tsgo $TSGO_VERSION"; fi
tmp="$(mktemp -d)"; trap 'rm -rf "$tmp"' EXIT

fail=0
# The pins.
for f in templates/*/package.json templates/archetypes/*/package.json examples/*/package.json; do
  if grep -q '"typescript":' "$f"; then
    og_err "check-conformance: $f declares typescript; og check (tsrs) is a project's checker"; fail=1
  fi
done
while read -r v; do
  if [ "$v" != "$TSRS_VERSION" ]; then
    og_err "check-conformance: packages/engine/package.json pins @tsrs/* $v; scripts/install-tools.sh TSRS_VERSION is $TSRS_VERSION"; fail=1
  fi
done < <(sed -n 's/^ *"@tsrs\/[^"]*": "\(.*\)",\{0,1\}$/\1/p' packages/engine/package.json)

if [ "$#" -gt 0 ]; then
  configs=("$@")
else
  # A template resolves the engine through node_modules in a created
  # project; here, as og check does, through paths to packages/engine.
  configs=(packages/engine/tsconfig.json packages/engine/tsconfig.runtime.json
    packages/engine/tsconfig.templates.json)
  for t in templates/*/ templates/archetypes/*/; do
    [ -f "$t/project.ogt" ] || continue
    t="${t%/}"; w="$tmp/wrap/$t"
    mkdir -p "$w"
    cat > "$w/tsconfig.json" <<EOF
{ "extends": "$OG_ROOT/$t/tsconfig.json",
  "compilerOptions": { "paths": {
    "@origingame/engine": ["$OG_ROOT/packages/engine/types/index.d.ts"],
    "@origingame/engine/playtest": ["$OG_ROOT/packages/engine/src/playtest.ts"] } } }
EOF
    configs+=("$w/tsconfig.json")
  done
  for e in brasswick hollowbell kestrel-stair lantern-vale meridian-loop; do configs+=("examples/$e/tsconfig.json"); done
  configs+=(scripts/tsrs-corpus/tsconfig.json)
fi

args=(--noEmit --incremental false --pretty false --checkers 4)
for c in "${configs[@]}"; do
  name="$c"; case "$c" in "$tmp"/wrap/*) name="${c#"$tmp"/wrap/}"; name="${name%/tsconfig.json}" ;; esac
  "${tsgo[@]}" -p "$c" "${args[@]}" > "$tmp/tsgo.txt" 2> "$tmp/tsgo.err"; a=$?
  "$TSRS" -p "$c" "${args[@]}" --checkerAssignment go --noLazyMembers > "$tmp/tsrs.txt" 2> "$tmp/tsrs.err"; b=$?
  # Exit 0-2 is a finished check (2 is TypeScript's "diagnostics, outputs
  # skipped"); 3 and above, or a signal, is the tool failing.
  if [ "$a" -gt 2 ] || [ "$b" -gt 2 ]; then
    og_err "check-conformance: $name: a checker failed (tsgo exit $a, tsrs exit $b)"
    cat "$tmp/tsgo.txt" "$tmp/tsgo.err" "$tmp/tsrs.txt" "$tmp/tsrs.err" >&2
    fail=1; continue
  fi
  python3 -I - "$name" "$tmp/tsgo.txt" "$tmp/tsrs.txt" <<'PY' || fail=1
import re, sys
name, go_path, rs_path = sys.argv[1:4]
# `<file>(<line>,<col>): error TS<code>: <message>`, `error TS<code>: <message>`,
# and indented continuation lines that belong to the diagnostic above.
head = re.compile(r"^(?:(.+)\((\d+),(\d+)\): )?(?:error|warning|message) TS(\d+): (.*)$")
def parse(path):
    out, last = {}, None
    for line in open(path, encoding="utf-8", errors="replace"):
        line = line.rstrip("\n")
        if not line:
            continue
        m = head.match(line)
        if m and not line.startswith(" "):
            key = (m[1] or "", int(m[2] or 0), int(m[3] or 0), int(m[4]))
            out.setdefault(key, []).append(m[5])
            last = key
        elif last is not None:
            out[last][-1] += "\n" + line.strip()
    return out
go, rs = parse(go_path), parse(rs_path)
missed = sorted(k for k in go if k not in rs)
extra = sorted(k for k in rs if k not in go)
differ = sorted(k for k in go if k in rs and go[k] != rs[k])
fmt = lambda k: f"{k[0] or '<global>'}({k[1]},{k[2]}) TS{k[3]}"
for k in extra:
    print(f"diff {name}: tsrs only: {fmt(k)}: {rs[k][0]}")
for k in differ:
    print(f"diff {name}: message differs at {fmt(k)}:\n  tsgo: {go[k][0]}\n  tsrs: {rs[k][0]}")
for k in missed:
    print(f"MISS {name}: {fmt(k)}: {go[k][0]}", file=sys.stderr)
total = sum(len(v) for v in go.values())
if not go and not rs:
    print(f"ok   {name}: both clean")
else:
    print(f"{'FAIL' if missed else 'ok  '} {name}: tsgo {total} diagnostic(s); tsrs missed {len(missed)}, "
          f"reported {len(extra)} more, {len(differ)} with another message")
sys.exit(1 if missed else 0)
PY
done

if [ "$fail" = 0 ]; then og_say "PASS check-conformance (tsrs $TSRS_VERSION against $against)"; exit 0; fi
og_err "FAIL check-conformance"
exit 1
