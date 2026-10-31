#!/usr/bin/env bash
# The release path (docs/release.md §3): one version across the four
# packages, release tarballs, the install test, and the publish.
#
#   scripts/release.sh check                    every occurrence of the version equals packages/engine's
#   scripts/release.sh bump <version>           the golden replays (scripts/replay-gate.ts), then write the version everywhere (scripts/release-bump.ts), Cargo.lock through cargo update
#   scripts/release.sh pack                     OG_RELEASE=1 packages/engine/scripts/pack.sh --linux-x64 (the engine with its knowledge/,
#                                               this machine's platform package and linux-x64's), then @origingame/create-game:
#                                               dist/release/<version>/*.tgz and manifest.json (sizes, sha256, the commit built from).
#                                               Since 0.6.0 this is the release workflow's job (.github/workflows/release.yml: the wasm
#                                               and the linux-x64 addon on one Linux runner, the darwin-arm64 and win32-x64 addons on
#                                               their runners, `pack.sh --native`), not a machine's; by hand it still packs here
#   scripts/release.sh manifest <dir>           write <dir>/manifest.json over the tarballs in <dir> (what pack ends with; the workflow's
#                                               assemble job runs it over every job's tarballs)
#   scripts/release.sh fetch <run-id>           download the release workflow run's `release-<version>` artifact (gh run download) into
#                                               dist/release/<version>/, where verify and publish read it
#   scripts/release.sh verify [--registry] [--host-only] [--container-only] [--instabox]
#                                               the install test (tests/install/run.ts) from the tarballs (or, --registry, from npm) on this
#                                               machine and in the linux-x64 container; --instabox runs it once more inside an InstaBox
#                                               machine (tests/install/instabox.ts, the development sandbox; INSTABOX_API_KEY, billed by
#                                               the minute); records each pass in manifest.json
#   scripts/release.sh publish [--dry-run] [--otp <code>]
#                                               npm publish each tarball (the three platform packages, the win32-x64 one copied in from
#                                               the Windows host's pack, then engine, create-game), tag v<version>;
#                                               refuses a dirty tree, a manifest not verified on both platforms, a HEAD whose packaged inputs
#                                               (crates, packages, templates, Cargo.lock, the wasm and tool scripts) differ from the manifest's commit, a version on the registry with other bytes, or no npm login. Resumable:
#                                               a package already published with these bytes is skipped. npm's two-factor prompt (a URL
#                                               to open, or --otp) is answered by the person running it, unless NPM_TOKEN is set (the
#                                               project secret: a granular access token with publish rights and 2FA bypass), which
#                                               publishes without a login or a prompt (a private userconfig for the run)
#   scripts/release.sh dev <run-id|dir>         publish a dev build (docs/release.md §1a): the tarballs of a `release.yml -f dev=true`
#                                               run (fetched, or a directory holding them) under npm's `dev` dist-tag; no verify, no git
#                                               tag, `latest` untouched. NPM_TOKEN or an npm login, as publish
#
# pack holds the gate lock through pack.sh (a release build compiles beside
# nothing) and refuses a dev native/ as pack.sh does. publish is an outward
# action and runs only by hand, never from a gate or the wave's end
# (scripts/wave-end.sh's release stage is pack and verify).
set -u -o pipefail
# shellcheck source-path=SCRIPTDIR
. "$(dirname "$0")/lib.sh"
cd "$OG_ROOT" || exit 1

usage() { awk 'NR>1 && !/^#/ {exit} NR>1 {sub(/^# ?/, ""); print}' "$0"; }
verb="${1:-}"; shift || true

version() { bun scripts/release-bump.ts --print; }
out_dir() { echo "$OG_ROOT/dist/release/$1"; }

cmd_check() { bun scripts/release-bump.ts --check; }

cmd_bump() {
  local v="${1:?scripts/release.sh bump <version>}"
  # The golden replays first (docs/release.md §3): their hashes unchanged,
  # or changelog.json's unreleased.simulation says "changed", before the
  # bump moves unreleased into the version's entry.
  bun scripts/replay-gate.ts || { og_err "release: the golden replays diverged (scripts/replay-gate.ts); set unreleased.simulation to \"changed\" in packages/engine/changelog.json if that is meant, and re-record with --update"; return 1; }
  bun scripts/release-bump.ts "$v" || return 1
  cargo update --workspace --offline -q || { og_err "release: cargo update --workspace failed (Cargo.lock)"; return 1; }
  # tools/wasm-determinism is its own workspace with its own lock on the
  # engine's crates by path; left at the old version, the gate's
  # determinism step rewrites it (0.3.0 left it at 0.2.0 until 2026-10-24).
  (cd tools/wasm-determinism && cargo update --workspace --offline -q) || { og_err "release: cargo update in tools/wasm-determinism failed (its Cargo.lock)"; return 1; }
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
  cmd_manifest "$out" || return 1
  og_say "release: packed $v (${out#"$OG_ROOT"/}/manifest.json)"
}

# The manifest over a directory of tarballs: the engine, create-game and at
# least one platform package (pack on a machine gives its own and linux-x64's;
# the workflow's web job gives linux-x64's and its assemble job every one).
cmd_manifest() {
  local out="${1:?scripts/release.sh manifest <dir>}" v
  out="$(cd "$out" 2>/dev/null && pwd)" || { og_err "release: no directory $1"; return 1; }
  v="$(version)"
  local n tgz platforms
  n=0; platforms=0
  for tgz in "$out"/*.tgz; do
    [ -e "$tgz" ] || continue
    n=$((n + 1))
    case "$(basename "$tgz")" in origingame-engine-*-"$v".tgz) platforms=$((platforms + 1)) ;; esac
  done
  [ -f "$out/origingame-engine-$v.tgz" ] && [ -f "$out/origingame-create-game-$v.tgz" ] && [ "$platforms" -ge 1 ] \
    || { og_err "release: $out needs origingame-engine-$v.tgz, origingame-create-game-$v.tgz and a platform package; it has $n tarballs:"; ls "$out" >&2; return 1; }
  # shellcheck disable=SC2016 # the bun script takes its arguments from argv
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
}

# The workflow run's artifact, into the directory pack would have filled, so
# verify and publish go on as before. The run's manifest names the commit the
# runners checked out; publish holds HEAD to it as to a local pack's.
cmd_fetch() {
  local run="${1:?scripts/release.sh fetch <run-id>}" v out
  v="$(version)"; out="$(out_dir "$v")"
  og_need gh || return 1
  rm -rf "$out"; mkdir -p "$out"
  gh run download "$run" -n "release-$v" -D "$out" || { og_err "release: gh run download $run -n release-$v failed (is the run a release.yml run of $v that finished?)"; return 1; }
  [ -f "$out/manifest.json" ] || { og_err "release: the artifact has no manifest.json"; ls "$out" >&2; return 1; }
  # Verification is this machine's to do: the artifact's manifest carries none.
  # shellcheck disable=SC2016 # the bun script takes its argument from argv
  bun -e 'const f=process.argv[1],fs=require("node:fs");const m=JSON.parse(fs.readFileSync(f,"utf8"));m.verified={};fs.writeFileSync(f,JSON.stringify(m,null,2)+"\n");for(const x of m.files)console.log(`${x.name}  ${x.bytes} bytes  sha256 ${x.sha256}`);console.log(`commit ${m.commit}${m.dirty?" (dirty)":""}`)' "$out/manifest.json" || return 1
  og_say "release: fetched run $run into ${out#"$OG_ROOT"/}/; now scripts/release.sh verify"
}

cmd_verify() {
  local v out
  v="$(version)"; out="$(out_dir "$v")"
  [ -f "$out/manifest.json" ] || { og_err "release: no $out/manifest.json; run scripts/release.sh pack first"; return 1; }
  local instabox=0 registry=0 rest=()
  for a in "$@"; do
    case "$a" in
      --instabox) instabox=1 ;;
      --registry) registry=1; rest+=("$a") ;;
      *) rest+=("$a") ;;
    esac
  done
  bun tests/install/run.ts --from "$out" --version "$v" --manifest "$out/manifest.json" "${rest[@]}" || return 1
  if [ "$instabox" -eq 1 ]; then
    if [ "$registry" -eq 1 ]; then
      bun tests/install/instabox.ts --version "$v" --registry --manifest "$out/manifest.json"
    else
      bun tests/install/instabox.ts --version "$v" --from "$out" --manifest "$out/manifest.json"
    fi
  fi
}

cmd_publish() {
  local dry=0 v out otp=""
  while [ $# -gt 0 ]; do
    case "$1" in
      --dry-run) dry=1 ;;
      --otp) otp="${2:?--otp <code>}"; shift ;;
      *) og_err "release: publish takes --dry-run and --otp <code>"; return 2 ;;
    esac
    shift
  done
  v="$(version)"; out="$(out_dir "$v")"
  [ -f "$out/manifest.json" ] || { og_err "release: no $out/manifest.json; run pack and verify first"; return 1; }
  local status
  # shellcheck disable=SC2016 # the bun script takes its arguments from argv
  status="$(bun -e '
    const m = JSON.parse(require("node:fs").readFileSync(process.argv[1], "utf8"));
    const problems = [];
    if (m.dirty) problems.push("packed from a dirty tree");
    if (!m.verified?.host) problems.push("not verified on this machine (scripts/release.sh verify)");
    if (!m.verified?.container) problems.push("not verified in the linux-x64 container (scripts/release.sh verify)");
    // The tarballs are a function of these paths: a later commit outside them (docs, tests/evidence) keeps the manifest valid.
    const git = (...a) => require("node:child_process").execFileSync("git", a, { encoding: "utf8" }).trim();
    const head = git("rev-parse", "HEAD");
    if (m.commit !== head) {
      let ancestor = true;
      try { git("merge-base", "--is-ancestor", m.commit, head); } catch { ancestor = false; }
      if (!ancestor) problems.push(`packed from ${m.commit.slice(0, 12)}, which is not an ancestor of HEAD ${head.slice(0, 12)}`);
      else {
        const inputs = ["crates", "Cargo.toml", "Cargo.lock", "rust-toolchain.toml", "packages", "templates", ".cargo", "scripts/wasm-crates.txt", "scripts/budgets.txt", "scripts/wasm-opt.sh", "scripts/wasm-lanes.sh", "scripts/install-tools.sh", "scripts/lib.sh", "scripts/lock.sh", "scripts/release-bump.ts"];
        const changed = git("diff", "--name-only", m.commit, head, "--", ...inputs);
        if (changed) problems.push(`packed from ${m.commit.slice(0, 12)}; since then these inputs of the packages changed (pack again):\n    ${changed.split("\n").join("\n    ")}`);
      }
    }
    if (require("node:child_process").execFileSync("git", ["status", "--porcelain"], { encoding: "utf8" }).trim()) problems.push("the working tree is dirty");
    console.log(problems.join("\n"));
  ' "$out/manifest.json")"
  if [ -n "$status" ]; then og_err "release: publish refused:"; printf '%s\n' "$status" | sed 's/^/  /' >&2; return 1; fi
  # NPM_TOKEN (the project secret, an npm granular access token with publish
  # rights on @origingame and 2FA bypass) stands in for npm login and the
  # one-time password: it goes into a private userconfig for this run, never
  # into the checkout's .npmrc or the log.
  if [ -n "${NPM_TOKEN:-}" ]; then
    local rc; rc="$(mktemp)" || return 1
    printf '//registry.npmjs.org/:_authToken=%s\n' "$NPM_TOKEN" > "$rc"
    export NPM_CONFIG_USERCONFIG="$rc"
    trap 'rm -f "$NPM_CONFIG_USERCONFIG"' RETURN
  fi
  local who
  who="$(npm whoami 2>/dev/null)" || { og_err "release: not logged in to npm (npm login, or NPM_TOKEN); publish needs the @origingame scope"; return 1; }
  og_say "release: publishing $v as $who"
  # Resumable: a package already on the registry at this version is skipped
  # when its bytes are these (npm's sha512 integrity), refused when they differ.
  # Platform packages first (the engine's optionalDependencies name them at this
  # version); the win32-x64 tarball is packed on the Windows host and copied into
  # dist/release/<v>/ beside the Mac's (tests/evidence/release/windows-x64.md).
  local pkgs="engine-darwin-arm64 engine-linux-x64 engine-win32-x64 engine create-game" name tgz published local_sum
  for name in $pkgs; do
    tgz="$out/origingame-$name-$v.tgz"
    [ -f "$tgz" ] || { og_err "release: no $tgz"; return 1; }
    published="$(npm view "@origingame/$name@$v" dist.integrity 2>/dev/null || true)"
    if [ -n "$published" ]; then
      local_sum="sha512-$(openssl dgst -sha512 -binary "$tgz" | base64 | tr -d '\n')"
      if [ "$published" = "$local_sum" ]; then og_say "release: @origingame/$name@$v is already on the registry with these bytes; skipped"; continue; fi
      og_err "release: @origingame/$name@$v is already on the registry with other bytes; bump the version"; return 1
    fi
    if [ "$dry" = 1 ]; then og_say "+ npm publish --access public $tgz (dry run)"; continue; fi
    og_say "+ npm publish --access public ${tgz#"$OG_ROOT"/}"
    # shellcheck disable=SC2086 # --otp is one word or none
    npm publish --access public ${otp:+--otp "$otp"} "$tgz" || { og_err "release: npm publish of @origingame/$name failed (a one-time password: run this in your terminal and open the URL npm prints, or pass --otp <code>); run publish again to continue, the packages already published are skipped"; return 1; }
  done
  if [ "$dry" = 1 ]; then og_say "release: dry run; nothing published"; return 0; fi
  if git rev-parse -q --verify "refs/tags/v$v" >/dev/null; then
    if [ "$(git rev-parse "v$v^{commit}")" = "$(git rev-parse HEAD)" ]; then og_say "release: tag v$v is already at HEAD"
    else og_err "release: tag v$v exists at $(git rev-parse --short "v$v^{commit}"), not at HEAD; move it by hand if that is wanted"; fi
  else
    git tag -a "v$v" -m "Origin Engine $v" && og_say "release: tagged v$v (push it: git push origin v$v)"
  fi
  og_say "release: published $v. Install: bun add @origingame/engine@$v ; npm create @origingame/game@$v my-game"
  og_say "release: now scripts/release.sh verify --registry"
}

# A dev build: what a pin moves to while a defect is tested (docs/release.md
# §1a). The version comes from the tarballs (the workflow wrote it in its
# checkout), not from this tree.
cmd_dev() {
  local src="${1:?scripts/release.sh dev <run-id|dir>}" out
  if [ -d "$src" ]; then out="$(cd "$src" && pwd)"
  else
    og_need gh || return 1
    out="$(mktemp -d)" || return 1
    gh run download "$src" -p 'release-*' -D "$out" || { og_err "release: gh run download $src failed (a finished release.yml run with -f dev=true?)"; return 1; }
    out="$(dirname "$(find "$out" -name manifest.json | head -1)")"
  fi
  local v
  v="$(bun -e 'console.log(JSON.parse(require("node:fs").readFileSync(process.argv[1],"utf8")).version)' "$out/manifest.json")" || { og_err "release: no manifest.json in $out"; return 1; }
  case "$v" in *-dev.*) ;; *) og_err "release: $v is not a dev build; a release goes through verify and publish"; return 1 ;; esac
  if [ -n "${NPM_TOKEN:-}" ]; then
    local rc; rc="$(mktemp)" || return 1
    printf '//registry.npmjs.org/:_authToken=%s\n' "$NPM_TOKEN" > "$rc"
    export NPM_CONFIG_USERCONFIG="$rc"
    trap 'rm -f "$NPM_CONFIG_USERCONFIG"' RETURN
  fi
  npm whoami >/dev/null 2>&1 || { og_err "release: not logged in to npm (NPM_TOKEN)"; return 1; }
  local name tgz
  for name in engine-darwin-arm64 engine-linux-x64 engine-win32-x64 engine create-game; do
    tgz="$out/origingame-$name-$v.tgz"
    [ -f "$tgz" ] || { og_err "release: no $tgz"; return 1; }
    if [ -n "$(npm view "@origingame/$name@$v" version 2>/dev/null)" ]; then og_say "release: @origingame/$name@$v is already published; skipped"; continue; fi
    npm publish --access public --tag dev "$tgz" || { og_err "release: npm publish of @origingame/$name@$v failed; run dev again to continue"; return 1; }
  done
  og_say "release: published dev build $v (npm dist-tag dev). Pin it: \"@origingame/engine\": \"$v\""
}

case "$verb" in
  check) cmd_check ;;
  dev) cmd_dev "$@" ;;
  bump) cmd_bump "$@" ;;
  pack) cmd_pack ;;
  manifest) cmd_manifest "$@" ;;
  fetch) cmd_fetch "$@" ;;
  verify) cmd_verify "$@" ;;
  publish) cmd_publish "$@" ;;
  -h|--help|help|"") usage; [ -n "$verb" ] || exit 2 ;;
  *) og_err "release: unknown verb '$verb'"; usage >&2; exit 2 ;;
esac
