#!/usr/bin/env bash
# The release path (docs/release.md §3): one version across the packages,
# the download channel's files (per target an archive of the dist layout,
# manifest.json, SHA256SUMS and its minisign signature), the install test,
# and the publish: an upload to the platform, which origingame.ai/downloads/engine/
# serves. Nothing goes to npm any more (the owner, 2026-11-05).
#
#   scripts/release.sh check                    every occurrence of the version equals packages/engine's
#   scripts/release.sh bump <version>           the golden replays (scripts/replay-gate.ts), then write the version everywhere (scripts/release-bump.ts,
#                                               which folds changelog/unreleased/ into changelog.json first), Cargo.lock through cargo update
#   scripts/release.sh fold                     fold changelog/unreleased/'s fragments into changelog.json's unreleased (scripts/changelog.ts);
#                                               run it at any time between releases, and commit the result
#   scripts/release.sh pack [<version>]         the checkout's version (no argument): OG_RELEASE=1 packages/engine/scripts/pack.sh --linux-x64
#                                               (the engine with its knowledge/, this machine's platform package and linux-x64's), then
#                                               create-game and design, as npm-shaped tarballs in dist/release/<version>/ with NOTES.md
#                                               (the release notes, from changelog.json), then `archive`.
#                                               Run on the Mac (docs/release.md §3); studio-win adds win32-x64's tarball (`build.sh --native`,
#                                               `pack.sh --native`, copied into dist/release/<version>/), then `archive` again.
#                                               `pack <version>` (a version npm carries: every one from 0.5.0, and 0.7.8-dev.17): its
#                                               published tarballs, downloaded from the registry with their integrity checked, then
#                                               archived: the bytes that version shipped with (the backfill, docs/release.md §3)
#   scripts/release.sh archive [<version>]      scripts/release-dist.ts over dist/release/<version>/: an archive per target whose platform
#                                               tarball is there, manifest.json, SHA256SUMS, and SHA256SUMS.minisig when
#                                               OG_ENGINE_MINISIGN_KEY (the project secret) is set; release.json is the local record
#   scripts/release.sh sign [<version>]         manifest.json, a release's install.sh and install.ps1, SHA256SUMS and its signature
#                                               again over the archives in place (OG_ENGINE_MINISIGN_KEY); the install test's passes stand
#   scripts/release.sh fetch <run-id>           download the release workflow run's `release-<version>` artifact (gh run download) into
#                                               dist/release/<version>/, then `archive`
#   scripts/release.sh verify [<version>] [--host-only] [--container-only] [--core] [--instabox]
#                                               the install test (tests/install/run.ts) from the archives on this machine and in the
#                                               linux-x64 container; --instabox runs it once more inside an InstaBox machine
#                                               (tests/install/instabox.ts; INSTABOX_API_KEY, billed by the minute); --core runs the steps
#                                               every version has (a backfill); records each pass in release.json
#   scripts/release.sh publish [<version>] [--dry-run]
#                                               upload SHA256SUMS, its signature and every file it lists (the three archives, manifest.json,
#                                               a release's install.sh and install.ps1) to the platform, then publish there
#                                               (scripts/release-publish.ts; OG_PLATFORM_ENGINE_UPLOAD_TOKEN); refuses a dirty pack, a missing
#                                               target, an unsigned SHA256SUMS or one that does not verify with docs/release.md's public
#                                               key, an archive not verified on both platforms, and (the checkout's version) a HEAD whose
#                                               packaged inputs differ from the revision packed. A version already published with the
#                                               same SHA256SUMS is reported and left; any other bytes are refused
#   scripts/release.sh dev <run-id|dir>         a dev build (docs/release.md §1a): the tarballs of a `release.yml -f dev=true` run
#                                               (fetched, or a directory holding them), archived, signed and published (channel dev)
#                                               at the run's commit; no verify
#
# pack holds the gate lock through pack.sh (a release build compiles beside
# nothing) and refuses a dev native/ as pack.sh does. publish is an outward
# action and runs only by hand, never from a gate.
set -u -o pipefail
# shellcheck source-path=SCRIPTDIR
. "$(dirname "$0")/lib.sh"
cd "$OG_ROOT" || exit 1

usage() { awk 'NR>1 && !/^#/ {exit} NR>1 {sub(/^# ?/, ""); print}' "$0"; }
verb="${1:-}"; shift || true

REPO=originai-game/origin-engine
TRIPLES="x86_64-unknown-linux-gnu aarch64-apple-darwin x86_64-pc-windows-msvc"
checkout_version() { bun scripts/release-bump.ts --print; }
out_dir() { echo "$OG_ROOT/dist/release/$1"; }
# The version a verb names: its first argument when that is a version, else the checkout's.
pick_version() {
  case "${1:-}" in [0-9]*.[0-9]*.[0-9]*) printf '%s\n' "$1" ;; *) checkout_version ;; esac
}
# The release public key docs/release.md states (packages/launcher/build.ts's pinnedKey);
# OG_ENGINE_MINISIGN_PUBKEY replaces it for a test key (the platform checks with its own pin).
pinned_key() { [ -z "${OG_ENGINE_MINISIGN_PUBKEY:-}" ] || { printf '%s\n' "$OG_ENGINE_MINISIGN_PUBKEY"; return 0; }; bun -e 'const { pinnedKey } = await import(process.argv[1]); const k = pinnedKey(); if (!k) process.exit(1); console.log(k)' "$OG_ROOT/packages/launcher/build.ts"; }

cmd_check() { bun scripts/release-bump.ts --check; }

cmd_bump() {
  local v="${1:?scripts/release.sh bump <version>}"
  bun scripts/release-bump.ts "$v" || return 1
  cargo update --workspace --offline -q || { og_err "release: cargo update --workspace failed (Cargo.lock)"; return 1; }
  og_say "release: bumped to $v (changelog/unreleased/ folded into changelog.json); Cargo.lock updated"
}

cmd_pack() {
  local v out
  if [ -n "${1:-}" ]; then
    # A version npm carries (published before 2026-11-05): its tarballs from the registry.
    v="$1"; out="$(out_dir "$v")"
    og_say "release: archiving $v's published packages"
    mkdir -p "$out"
    bun scripts/release-dist.ts "$out" --version "$v" --registry || return 1
    return 0
  fi
  v="$(checkout_version)"; out="$(out_dir "$v")"
  cmd_check || return 1
  rm -rf "$out"; mkdir -p "$out"
  bun scripts/changelog.ts notes "$v" > "$out/NOTES.md" || { og_err "release: changelog.json has no $v entry for the release notes; run scripts/release.sh bump $v"; return 1; }
  og_say "release: packing $v into ${out#"$OG_ROOT"/}/"
  (cd packages/engine && sh scripts/pack.sh --linux-x64) || { og_err "release: packages/engine/scripts/pack.sh failed"; return 1; }
  cp packages/engine/dist/pack/*.tgz "$out/" || return 1
  (cd packages/create && bun scripts/pack-template.ts >/dev/null && bun pm pack --destination "$out" --quiet --ignore-scripts >/dev/null) || { og_err "release: packing @origingame/create-game failed"; return 1; }
  (cd packages/design && bun pm pack --destination "$out" --quiet --ignore-scripts >/dev/null) || { og_err "release: packing @origingame/design failed"; return 1; }
  cmd_archive "$v" || return 1
  [ -f "$out/origingame-engine-win32-x64-$v.tgz" ] || og_say "release: now win32-x64 from studio-win (docs/release.md §3), copied into ${out#"$OG_ROOT"/}/, then scripts/release.sh archive"
}

cmd_archive() {
  local v out
  v="$(pick_version "${1:-}")"; out="$(out_dir "$v")"
  [ -f "$out/origingame-engine-$v.tgz" ] || { og_err "release: no ${out#"$OG_ROOT"/}/origingame-engine-$v.tgz; run scripts/release.sh pack $v"; return 1; }
  bun scripts/release-dist.ts "$out" --version "$v"
}

# The workflow run's artifact (the tarballs), into the directory pack would
# have filled, then archived here, where the signing key is.
cmd_fetch() {
  local run="${1:?scripts/release.sh fetch <run-id>}" v out
  v="$(checkout_version)"; out="$(out_dir "$v")"
  og_need gh || return 1
  rm -rf "$out"; mkdir -p "$out"
  gh run download "$run" -n "release-$v" -D "$out" || { og_err "release: gh run download $run -n release-$v failed (is the run a release.yml run of $v that finished?)"; return 1; }
  bun scripts/release-dist.ts "$out" --version "$v" --revision "$(gh run view "$run" -R "$REPO" --json headSha -q .headSha)"
}

cmd_verify() {
  local v out
  v="$(pick_version "${1:-}")"; out="$(out_dir "$v")"
  case "${1:-}" in [0-9]*) shift ;; esac
  [ -f "$out/release.json" ] || { og_err "release: no ${out#"$OG_ROOT"/}/release.json; run scripts/release.sh pack first"; return 1; }
  local instabox=0 rest=() key
  for a in "$@"; do
    case "$a" in
      --instabox) instabox=1 ;;
      *) rest+=("$a") ;;
    esac
  done
  key="$(pinned_key 2>/dev/null || true)"
  bun tests/install/run.ts --from "$out" --version "$v" --record "$out/release.json" ${key:+--pubkey "$key"} "${rest[@]}" || return 1
  if [ "$instabox" -eq 1 ]; then
    bun tests/install/instabox.ts --version "$v" --from "$out" --record "$out/release.json" ${key:+--pubkey "$key"} "${rest[@]}"
  fi
}

cmd_publish() {
  local dry=() arg=""
  for a in "$@"; do
    case "$a" in
      --dry-run) dry=(--dry-run) ;;
      [0-9]*) arg="$a" ;;
      *) og_err "release: publish takes [<version>] and --dry-run"; return 2 ;;
    esac
  done
  local v out
  v="$(pick_version "$arg")"; out="$(out_dir "$v")"
  [ -f "$out/release.json" ] || { og_err "release: no ${out#"$OG_ROOT"/}/release.json; run pack and verify first"; return 1; }
  og_need minisign || return 1
  local key; key="$(pinned_key)" || { og_err "release: docs/release.md states no release public key (§1)"; return 1; }
  local status
  # shellcheck disable=SC2016 # the bun script takes its arguments from argv
  status="$(bun -e '
    const fs = require("node:fs");
    const [file, triples, dev] = process.argv.slice(1);
    const m = JSON.parse(fs.readFileSync(file, "utf8"));
    const problems = [];
    if (m.dirty) problems.push("packed from a dirty tree");
    const missing = triples.split(" ").filter((t) => !m.targets.includes(t));
    if (missing.length) problems.push(`no archive for ${missing.join(", ")} (scripts/release.sh archive once the win32-x64 tarball is in)`);
    if (!m.signed) problems.push("SHA256SUMS is unsigned (OG_ENGINE_MINISIGN_KEY, then scripts/release.sh archive)");
    if (dev !== "1") {
      if (!m.verified?.host) problems.push("not verified on this machine (scripts/release.sh verify)");
      if (!m.verified?.container) problems.push("not verified in the linux-x64 container (scripts/release.sh verify)");
    }
    if (m.source === "build" && !m.version.includes("-dev.")) {
      // The archives are a function of these paths: a later commit outside them (docs, the former test sample) keeps the pack valid.
      const git = (...a) => require("node:child_process").execFileSync("git", a, { encoding: "utf8" }).trim();
      const head = git("rev-parse", "HEAD");
      if (m.revision !== head) {
        let ancestor = true;
        try { git("merge-base", "--is-ancestor", m.revision, head); } catch { ancestor = false; }
        if (!ancestor) problems.push(`packed from ${m.revision.slice(0, 12)}, which is not an ancestor of HEAD ${head.slice(0, 12)}`);
        else {
          const inputs = ["crates", "Cargo.toml", "Cargo.lock", "rust-toolchain.toml", "packages", "templates", ".cargo", "scripts/wasm-crates.txt", "scripts/budgets.txt", "scripts/wasm-lanes.sh", "scripts/release-wasm.sh", "scripts/gate/commands.sh", "scripts/size-gate.sh", "scripts/install-tools.sh", "scripts/lib.sh", "scripts/lock.sh", "scripts/release-bump.ts", "scripts/release-dist.ts"];
          const changed = git("diff", "--name-only", m.revision, head, "--", ...inputs);
          if (changed) problems.push(`packed from ${m.revision.slice(0, 12)}; since then these inputs changed (pack again):\n    ${changed.split("\n").join("\n    ")}`);
        }
      }
    }
    console.log(problems.join("\n"));
  ' "$out/release.json" "$TRIPLES" "${OG_RELEASE_DEV:-0}")"
  if [ -n "$status" ]; then og_err "release: publish refused:"; printf '%s\n' "$status" | sed 's/^/  /' >&2; return 1; fi
  minisign -V -q -P "$key" -m "$out/SHA256SUMS" -x "$out/SHA256SUMS.minisig" || { og_err "release: SHA256SUMS.minisig does not verify with docs/release.md's key $key"; return 1; }
  (cd "$out" && shasum -a 256 -c SHA256SUMS >/dev/null) || { og_err "release: a file does not match SHA256SUMS"; return 1; }
  bun scripts/release-publish.ts "$out" --version "$v" "${dry[@]}"
}

# Signs a packed version again (manifest.json, a release's installers, SHA256SUMS, the signature) over its
# archives in place, keeping the install test's passes: what a thread with OG_ENGINE_MINISIGN_KEY runs on
# archives packed and verified elsewhere.
cmd_sign() {
  local v; v="$(pick_version "${1:-}")"
  [ -n "${OG_ENGINE_MINISIGN_KEY:-}" ] || { og_err "release: no OG_ENGINE_MINISIGN_KEY (the project secret)"; return 1; }
  bun scripts/release-dist.ts "$(out_dir "$v")" --version "$v" --sign
}

# A dev build: what a pin moves to while a defect is tested (docs/release.md
# §1a). The version comes from the tarballs (the workflow wrote it in its
# checkout), not from this tree.
cmd_dev() {
  local src="${1:?scripts/release.sh dev <run-id|dir>}" tmp
  if [ -d "$src" ]; then tmp="$(cd "$src" && pwd)"
  else
    og_need gh || return 1
    tmp="$(mktemp -d)" || return 1
    gh run download "$src" -R "$REPO" -p 'release-*' -D "$tmp" || { og_err "release: gh run download $src failed (a finished release.yml run with -f dev=true?)"; return 1; }
    tmp="$(dirname "$(find "$tmp" -name 'origingame-engine-[0-9]*.tgz' | head -1)")"
  fi
  local v
  v="$(basename "$(find "$tmp" -maxdepth 1 -name 'origingame-engine-[0-9]*.tgz' | head -1)" .tgz)"; v="${v#origingame-engine-}"
  case "$v" in *-dev.*) ;; *) og_err "release: '$v' in $tmp is not a dev build; a release goes through pack, verify and publish"; return 1 ;; esac
  local out; out="$(out_dir "$v")"
  rm -rf "$out"; mkdir -p "$out"
  cp "$tmp"/origingame-*-"$v".tgz "$out/" || return 1
  local rev=()
  [ -d "$src" ] || rev=(--revision "$(gh run view "$src" -R "$REPO" --json headSha -q .headSha)")
  bun scripts/release-dist.ts "$out" --version "$v" "${rev[@]}" || return 1
  OG_RELEASE_DEV=1 cmd_publish "$v"
}

case "$verb" in
  check) cmd_check ;;
  dev) cmd_dev "$@" ;;
  bump) cmd_bump "$@" ;;
  fold) bun scripts/changelog.ts fold ;;
  pack) cmd_pack "$@" ;;
  archive) cmd_archive "$@" ;;
  sign) cmd_sign "$@" ;;
  fetch) cmd_fetch "$@" ;;
  verify) cmd_verify "$@" ;;
  publish) cmd_publish "$@" ;;
  -h|--help|help|"") usage; [ -n "$verb" ] || exit 2 ;;
  *) og_err "release: unknown verb '$verb'"; usage >&2; exit 2 ;;
esac
