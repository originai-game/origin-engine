#!/usr/bin/env bash
# check-conformance: tsrs (og check's checker) held to tsgo, the TypeScript
# compiler it ports (typescript@TSGO_VERSION, the constant in
# crates/og-cli/src/commands/check.rs), on the same configs with the same checker
# count. A tsrs version move runs it before it lands (docs/ci.md §Toolchain).
#
#   scripts/tsrs-diff.sh [CONFIG...]
#
# Without arguments: packages/engine's three tsconfigs, every template (with
# the paths og check gives a project without node_modules), every test
# fixture under tests/fixtures/ and the error corpus scripts/tsrs-corpus/ (22 files, one error a
# game author makes each). Each config runs once per checker (`-p CONFIG
# --noEmit --incremental false --pretty false --checkers 4`; tsrs adds
# `--checkerAssignment go --noLazyMembers`, the order closest to tsgo's) and
# each side's diagnostics become (file, line, col, code) keys:
#
#   - one tsgo reports and tsrs does not (a miss) fails, each listed;
#   - one tsrs reports and tsgo does not, or the same key with another
#     message, is printed (`diff` lines) and passes; re-check those with
#     the typescript 7.1.0-beta tsrs ports before calling them tsrs defects;
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

TSGO_VERSION="$(sed -n 's/^pub const TSGO_VERSION: &str = "\(.*\)";/\1/p' crates/og-cli/src/commands/check.rs)"
# shellcheck disable=SC2016 # the $ is sed's, matched literally
TSRS_VERSION="$(sed -n 's/^TSRS_VERSION="${TSRS_VERSION:-\(.*\)}"$/\1/p' scripts/install-tools.sh)"
[ -n "$TSGO_VERSION" ] || { og_err "check-conformance: no TSGO_VERSION in crates/og-cli/src/commands/check.rs"; exit 1; }
[ -n "$TSRS_VERSION" ] || { og_err "check-conformance: no TSRS_VERSION in scripts/install-tools.sh"; exit 1; }
TSRS="${TSRS:-tsrs}"
command -v "$TSRS" >/dev/null || { og_err "check-conformance: no tsrs; run scripts/install-tools.sh"; exit 1; }
if [ -n "${TSGO:-}" ]; then tsgo=("$TSGO"); against="$("$TSGO" --version)"
else og_need bun || exit 1; tsgo=(bun x --package "typescript@$TSGO_VERSION" tsc); against="tsgo $TSGO_VERSION"; fi
tmp="$(mktemp -d)"; trap 'rm -rf "$tmp"' EXIT

fail=0
# The pins.
for f in packages/create/template/package.json tests/fixtures/*/package.json; do
  [ -f "$f" ] || continue
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
  for t in packages/create/template/ tests/fixtures/*/; do
    [ -f "$t/og-project.json" ] || continue
    t="${t%/}"; w="$tmp/wrap/$t"
    mkdir -p "$w"
    cat > "$w/tsconfig.json" <<EOF
{ "extends": "$OG_ROOT/$t/tsconfig.json",
  "compilerOptions": { "paths": {
    "@origingame/engine": ["$OG_ROOT/packages/engine/types/index.d.ts"],
    "@origingame/engine/playtest": ["$OG_ROOT/packages/engine/src/playtest.ts"],
    "bun:test": ["$OG_ROOT/packages/engine/vendor/bun-types/test.d.ts"] } } }
EOF
    configs+=("$w/tsconfig.json")
  done
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
  bun - "$name" "$tmp/tsgo.txt" "$tmp/tsrs.txt" <<'TS' || fail=1
import { readFileSync } from "node:fs";
const [name, goPath, rsPath] = process.argv.slice(2);
// File-position and global diagnostics, followed by indented continuation lines.
const head = /^(?:(.+)\((\d+),(\d+)\): )?(?:error|warning|message) TS(\d+): (.*)$/;
function parse(path) {
  const out = new Map(); let last;
  for (const line of readFileSync(path,"utf8").split(/\r?\n/)) {
    if (!line) continue;
    const m = line.match(head);
    if (m && !line.startsWith(" ")) {
      const key = JSON.stringify([m[1]||"", Number(m[2]||0), Number(m[3]||0), Number(m[4])]);
      if (!out.has(key)) out.set(key, []);
      out.get(key).push(m[5]); last=key;
    } else if (last!==undefined) {
      const messages=out.get(last); messages[messages.length-1]+="\n"+line.trim();
    }
  }
  return out;
}
const sort = keys => keys.sort((a,b) => {
  const x=JSON.parse(a), y=JSON.parse(b);
  return (x[0]<y[0] ? -1 : x[0]>y[0] ? 1 : 0) || x[1]-y[1] || x[2]-y[2] || x[3]-y[3];
});
const go=parse(goPath), rs=parse(rsPath);
const missed=sort([...go.keys()].filter(k=>!rs.has(k))), extra=sort([...rs.keys()].filter(k=>!go.has(k)));
const differ=sort([...go.keys()].filter(k=>rs.has(k)&&JSON.stringify(go.get(k))!==JSON.stringify(rs.get(k))));
const fmt = key => {const k=JSON.parse(key); return `${k[0]||"<global>"}(${k[1]},${k[2]}) TS${k[3]}`;};
for(const k of extra) console.log(`diff ${name}: tsrs only: ${fmt(k)}: ${rs.get(k)[0]}`);
for(const k of differ) console.log(`diff ${name}: message differs at ${fmt(k)}:\n  tsgo: ${go.get(k)[0]}\n  tsrs: ${rs.get(k)[0]}`);
for(const k of missed) console.error(`MISS ${name}: ${fmt(k)}: ${go.get(k)[0]}`);
const total=[...go.values()].reduce((n,v)=>n+v.length,0);
if(!go.size&&!rs.size) console.log(`ok   ${name}: both clean`);
else console.log(`${missed.length ? "FAIL" : "ok  "} ${name}: tsgo ${total} diagnostic(s); tsrs missed ${missed.length}, reported ${extra.length} more, ${differ.length} with another message`);
process.exit(missed.length ? 1 : 0);
TS
done

if [ "$fail" = 0 ]; then og_say "PASS check-conformance (tsrs $TSRS_VERSION against $against)"; exit 0; fi
og_err "FAIL check-conformance"
exit 1
