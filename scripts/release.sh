#!/usr/bin/env bash
# The release path (docs/release.md §3): one version across the five
# packages, release tarballs, the install test, and the publish.
#
#   scripts/release.sh check                    every occurrence of the version equals packages/engine's
#   scripts/release.sh bump <version>           write the version everywhere (scripts/release-bump.ts), Cargo.lock through cargo update
#   scripts/release.sh pack                     OG_RELEASE=1 packages/engine/scripts/pack.sh --linux-x64, then @origingame/create-game and
#                                               @origingame/engine-knowledge: dist/release/<version>/*.tgz and manifest.json (sizes, sha256,
#                                               the commit built from)
#   scripts/release.sh verify [--registry] [--host-only] [--container-only]
#                                               the install test (tests/install/run.ts) from the tarballs (or, --registry, from npm) on this
#                                               machine and in the linux-x64 container; records the pass in manifest.json
#   scripts/release.sh publish [--dry-run]      npm publish each tarball (platform packages, engine, create-game, knowledge), tag v<version>;
#                                               refuses a dirty tree, a manifest not verified on both platforms, a HEAD that is not the
#                                               manifest's commit, a version already on the registry, or no npm login
#
# pack holds the gate lock through pack.sh (a release build compiles beside
# nothing) and refuses a dev native/ as pack.sh does. publish is an outward
# action and runs only by hand, never from a gate or the wave's end
# (scripts/wave-end.sh's release stage is pack and verify).
set -u -o pipefail
# shellcheck source-path=SCRIPTDIR
. "$(dirname "$0")/lib.sh"
cd "$OG_ROOT" || exit 1

usage() { sed -n '2,21p' "$0" | sed 's/^# \{0,1\}//'; }
verb="${1:-}"; shift || true

version() { bun scripts/release-bump.ts --print; }
out_dir() { echo "$OG_ROOT/dist/release/$1"; }

cmd_check() { bun scripts/release-bump.ts --check; }

cmd_bump() {
  local v="${1:?scripts/release.sh bump <version>}"
  bun scripts/release-bump.ts "$v" || return 1
  cargo update --workspace --offline -q || { og_err "release: cargo update --workspace failed (Cargo.lock)"; return 1; }
  if ! grep -q "^## $v" CHANGELOG.md; then
    og_err "release: CHANGELOG.md has no \"## $v\" heading; write the entry (one line per IDL member added, changed or removed)"
    return 1
  fi
  og_say "release: bumped to $v; Cargo.lock updated"
}

cmd_pack() {
  cmd_check || return 1
  local v out
  v="$(version)"; out="$(out_dir "$v")"
  og_need bun npm sha256sum shasum >/dev/null 2>&1 || true
  rm -rf "$out"; mkdir -p "$out"
  og_say "release: packing $v into ${out#"$OG_ROOT"/}/"
  (cd packages/engine && sh scripts/pack.sh --linux-x64) || { og_err "release: packages/engine/scripts/pack.sh failed"; return 1; }
  cp packages/engine/dist/pack/*.tgz "$out/" || return 1
  (cd packages/create && bun scripts/pack-template.ts >/dev/null && bun pm pack --destination "$out" --quiet --ignore-scripts >/dev/null) || { og_err "release: packing @origingame/create-game failed"; return 1; }
  (cd packages/knowledge && bun scripts/pack.ts >/dev/null && bun pm pack --destination "$out" --quiet --ignore-scripts >/dev/null) || { og_err "release: packing @origingame/engine-knowledge failed"; return 1; }
  local n
  n="$(ls "$out"/*.tgz | wc -l | tr -d ' ')"
  [ "$n" -eq 5 ] || { og_err "release: expected 5 tarballs in $out, found $n"; ls "$out"; return 1; }
  bun -e '
    const { readdirSync, readFileSync, statSync, writeFileSync } = require("node:fs");
    const { createHash } = require("node:crypto");
    const { execFileSync } = require("node:child_process");
    const [out, version] = process.argv.slice(1);
    const files = readdirSync(out).filter((f) => f.endsWith(".tgz")).sort().map((name) => ({
      name, bytes: statSync(`${out}/${name}`).size, sha256: createHash("sha256").update(readFileSync(`${out}/${name}`)).digest("hex"),
    }));
    const commit = execFileSync("git", ["rev-parse", "HEAD"], { encoding: "utf8" }).trim();
    const dirty = execFileSync("git", ["status", "--porcelain"], { encoding: "utf8" }).trim() !== "";
    writeFileSync(`${out}/manifest.json`, JSON.stringify({ version, commit, dirty, builtAt: new Date().toISOString(), files, verified: {} }, null, 2) + "\n");
    for (const f of files) console.log(`${f.name}  ${f.bytes} bytes  sha256 ${f.sha256}`);
    if (dirty) console.error("release: note: the working tree was dirty when these were packed; publish will refuse them until a clean tree is packed");
  ' "$out" "$v" || return 1
  og_say "release: packed $v (${out#"$OG_ROOT"/}/manifest.json)"
}

cmd_verify() {
  local v out
  v="$(version)"; out="$(out_dir "$v")"
  [ -f "$out/manifest.json" ] || { og_err "release: no $out/manifest.json; run scripts/release.sh pack first"; return 1; }
  bun tests/install/run.ts --from "$out" --version "$v" --manifest "$out/manifest.json" "$@"
}

cmd_publish() {
  local dry=0 v out
  [ "${1:-}" = --dry-run ] && dry=1
  v="$(version)"; out="$(out_dir "$v")"
  [ -f "$out/manifest.json" ] || { og_err "release: no $out/manifest.json; run pack and verify first"; return 1; }
  local status
  status="$(bun -e '
    const m = JSON.parse(require("node:fs").readFileSync(process.argv[1], "utf8"));
    const problems = [];
    if (m.dirty) problems.push("packed from a dirty tree");
    if (!m.verified?.host) problems.push("not verified on this machine (scripts/release.sh verify)");
    if (!m.verified?.container) problems.push("not verified in the linux-x64 container (scripts/release.sh verify)");
    const head = require("node:child_process").execFileSync("git", ["rev-parse", "HEAD"], { encoding: "utf8" }).trim();
    if (m.commit !== head) problems.push(`packed from ${m.commit.slice(0, 12)}, HEAD is ${head.slice(0, 12)}`);
    if (require("node:child_process").execFileSync("git", ["status", "--porcelain"], { encoding: "utf8" }).trim()) problems.push("the working tree is dirty");
    console.log(problems.join("\n"));
  ' "$out/manifest.json")"
  if [ -n "$status" ]; then og_err "release: publish refused:"; printf '%s\n' "$status" | sed 's/^/  /' >&2; return 1; fi
  local who
  who="$(npm whoami 2>/dev/null)" || { og_err "release: not logged in to npm (npm login); publish needs the @origingame scope"; return 1; }
  og_say "release: publishing $v as $who"
  local pkgs="engine-darwin-arm64 engine-linux-x64 engine create-game engine-knowledge" name tgz
  for name in $pkgs; do
    if npm view "@origingame/$name@$v" version >/dev/null 2>&1; then
      og_err "release: @origingame/$name@$v is already on the registry; bump the version"; return 1
    fi
  done
  for name in $pkgs; do
    tgz="$out/origingame-$name-$v.tgz"
    [ -f "$tgz" ] || { og_err "release: no $tgz"; return 1; }
    if [ "$dry" = 1 ]; then og_say "+ npm publish --access public $tgz (dry run)"; continue; fi
    og_say "+ npm publish --access public ${tgz#"$OG_ROOT"/}"
    npm publish --access public "$tgz" || { og_err "release: npm publish of @origingame/$name failed; the packages before it are published, finish by hand in this order: $pkgs"; return 1; }
  done
  if [ "$dry" = 1 ]; then og_say "release: dry run; nothing published"; return 0; fi
  git tag -a "v$v" -m "Origin Engine $v" && og_say "release: tagged v$v (push it: git push origin v$v)"
  og_say "release: published $v. Install: bun add @origingame/engine@$v ; npm create @origingame/game@$v my-game"
  og_say "release: now scripts/release.sh verify --registry"
}

case "$verb" in
  check) cmd_check ;;
  bump) cmd_bump "$@" ;;
  pack) cmd_pack ;;
  verify) cmd_verify "$@" ;;
  publish) cmd_publish "$@" ;;
  -h|--help|help|"") usage; [ -n "$verb" ] || exit 2 ;;
  *) og_err "release: unknown verb '$verb'"; usage >&2; exit 2 ;;
esac
