#!/usr/bin/env bash
# The release path (docs/release.md §3): one version across the four
# packages, release tarballs, the install test, and the publish.
#
#   scripts/release.sh check                    every occurrence of the version equals packages/engine's
#   scripts/release.sh bump <version>           write the version everywhere (scripts/release-bump.ts), Cargo.lock through cargo update
#   scripts/release.sh pack                     OG_RELEASE=1 packages/engine/scripts/pack.sh --linux-x64 (the engine with its knowledge/,
#                                               the two platform packages), then @origingame/create-game: dist/release/<version>/*.tgz and
#                                               manifest.json (sizes, sha256, the commit built from)
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
  local n
  n="$(ls "$out"/*.tgz | wc -l | tr -d ' ')"
  [ "$n" -eq 4 ] || { og_err "release: expected 4 tarballs in $out, found $n"; ls "$out"; return 1; }
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

case "$verb" in
  check) cmd_check ;;
  bump) cmd_bump "$@" ;;
  pack) cmd_pack ;;
  verify) cmd_verify "$@" ;;
  publish) cmd_publish "$@" ;;
  -h|--help|help|"") usage; [ -n "$verb" ] || exit 2 ;;
  *) og_err "release: unknown verb '$verb'"; usage >&2; exit 2 ;;
esac
