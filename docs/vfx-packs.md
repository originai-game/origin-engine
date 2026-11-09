# Effect packs for the asset library

Effect content (converted Effekseer samples, the Kenney-based starter sets,
the Pixel VFX Forge library) lives in the platform's asset library; the
engine's own packs ship with its releases, a few bundled and the rest
downloadable (§5). The engine owns the formats (`type
vfx`, `type vfx2d`, `type particles`, `type effect`), the `vfx` extension
kind, `og vfx add`, and the tool that prepares a pack: `scripts/vfx-pack.py`.
This is the contract agreed with the platform's importer
(`origingame` packages/asset-admin, `vfx:import`).

## 1. The pack

One directory, named as the pack, which `og vfx add <dir> [project]`
copies to `assets/vfx/<name>/` (hidden files skipped, symlinks refused, an
existing install never replaced). Its effects name their assets as
`vfx/<name>/<path>`.

`extension.json`: `format` 1, `kind` `"vfx"`, `name` (lowercase letters,
digits and `-`, unique in the library: `effekseer-<contributor>`,
`pixel-scenario`), `version` x.y.z, `description`, no `entry` or `types`,
and the catalog fields:

- `minEngine`: x.y.z, the first release whose cook and runtime read every
  line in the pack. `og vfx add` refuses a pack for a newer engine
  (`--any-engine` installs it anyway, for validation on unreleased builds);
  the platform imports it only once that release is published.
- `tags`: `{ "style": "stylised" | "realistic" | "pixel", "dim": "2d" | "3d",
  "category": [...], "element": [...] }`, the union of its effects' values.
- `keywords`: lowercase words.

## 2. assets.json

Beside `extension.json`, `format` 1:

- `engine`: `{ "version": <minEngine>, "revision": <the main revision it was made with> }`.
- `files`: every installed file (extension.json, README.md, each `.ogt`,
  recipe, texture, sheet and model; not assets.json, nothing under
  `preview/`) with `path`, `sha256`, `licence` (an SPDX id, or `"custom"`
  with `licenceText`), `source` (`url`, https; `file`, the original name,
  optional for authored files), `credit` and `derived` (how this file was
  made from the source). The importer refuses unlisted, missing or changed
  files.
- `effects`: one per effect a game spawns (`type vfx` or `vfx2d`, not its
  emitters or materials): `path`, `type`, `title`, `loop`,
  `durationSeconds`, `impact` (whether it sends one), `category`,
  `element` (optional: omitted when an effect has none), `keywords`.

The platform computes the pack's content hash (sha256 over the lines
`<path>\0<sha256>\n` sorted by path).

## 3. preview/

Inside the pack, stored by the library and never installed:

- `preview/<effect>.png`: six 960×540 frames in time order, the impact
  frame among them, 3 columns × 2 rows (2880×1080).
- `preview/<effect>.cost.json`: `{ "format": 1, "engine", "device",
  "peakFrameMs": { "total", "sim", "transparent" }, "peakParticles",
  "draws" }`. `total` is the 90th percentile of the effect's GPU frame
  time over the empty frames before it in its own timed run (from the fifth tick after the spawn: the first
  frames upload its textures once); `sim` the particle simulation pass;
  `transparent` the growth of the main pass, which draws blended surfaces,
  sprites and particles after the opaque scene. `peakParticles` is the most
  particle slots its systems held (`dump_render`'s `particles.capacity`,
  plus the live ones on the CPU path); `draws` the most draw calls of a
  frame.
- `preview/cover.png`: one 960×540 frame, the search thumbnail.

## 4. Making a pack

```sh
python3 scripts/vfx-pack.py manifest <pack> --spec <spec.json>   # assets.json, catalog fields
python3 scripts/vfx-pack.py preview <pack> [--cover <effect.ogt>] [--only <a.ogt,...>]  # preview/, timing into assets.json; --only re-renders those effects and keeps the rest
```

The spec names the provenance of each file by glob and each effect's
title, category, element and keywords (the script's docstring has its
shape). `preview` needs the native addon the gate builds
(`OG_GATE=1 sh packages/engine/scripts/build.sh --native` copies it after
`scripts/check.sh`) and takes its GPU locks itself
(scripts/lock.sh), each for one `og run`, so run it without a lock of your
own. It installs the pack into a scratch game, then:

1. measures each effect's length, loop and impact in one headless run (no
   GPU slot);
2. takes the six shots and render dumps a few effects per run, each run
   holding one GPU slot (`og_gpu_acquire`);
3. times each effect in its own short traced run, an empty scene (that
   run's baseline) and then the effect, holding every slot
   (`og_gpu_acquire_all`) for that run only and releasing them before the
   next effect.

All slots are held for one measured run, never across a pack.

## 5. The engine's packs: bundled and downloadable

The engine's own effect packs ship two ways (the coordinator, 2026-11-07):

- **Bundled**: a few, in `packages/engine/extensions/<pack>/` like the UI
  kits. They are in every engine archive's `engine/extensions/`, `og ui
  list` names them, and `og vfx add <pack>` copies one without a network.
- **Downloadable**: the rest, as plain Git files (no LFS) in
  `packages/vfx-packs/<pack>/`, outside the engine package and every engine
  archive. Each release carries one archive per pack,
  `<version>/vfx/<pack>.tar.gz` (docs/release.md §1), and `og vfx add <pack>`
  fetches it when no directory, bundled pack or installed pack has that name.

Bundled: `pixel-scenario` (the Pixel VFX Forge library, `vfx2d`) and
`effekseer-starter` (a curated set across the Effekseer contributors).
Downloadable: `effekseer-<contributor>`, every sample of one contributor.
A bundled pack's `preview/` stays in Git for the library and out of the
engine package (`!extensions/*/preview` in its `files`). `og api gen`
writes every pack's description, tags, keywords and effects into the
`effect-packs` guide (`crates/og-api/guides/effect-packs.md`, generated and
drift-checked), so `og api find heal` names the packs and effects that have one.

A pack is one or the other: `scripts/release-dist.ts` refuses an engine
archive with an `engine/extensions/<pack>` of a downloadable pack's name, or
anything of `packages/vfx-packs/`. The downloadable packs of a release are
the keys of its `manifest.json`'s `vfx`; `packages/vfx-packs/README.md`
lists the ones on `main`.

`og vfx add <pack> [project] [--any-engine]` for a downloadable pack, with
`<downloads>` `$ORIGIN_ENGINE_DOWNLOADS` (default
`https://origingame.ai/downloads/engine`, also a directory or `file://`; the
launcher's variable) and `<v>` the running engine's version:

1. downloads `<downloads>/<v>/SHA256SUMS` and `SHA256SUMS.minisig` and
   verifies the signature with the engine's key (docs/release.md §1;
   `$ORIGIN_ENGINE_PUBKEY` replaces it for a test channel), the launcher's
   method (`packages/engine/src/node/vfx-pack.ts`, the same code as
   `packages/launcher/og.ts`'s `verifyMinisign`);
2. downloads `manifest.json` and checks it against the signed list; the
   pack must be in its `vfx`, with the same sha256 as the list's
   `vfx/<pack>.tar.gz` line;
3. downloads the archive (at most 64 MiB) and checks its sha256;
4. unpacks it into a fresh temporary directory: only regular files and
   directories, every path under `<pack>/` with no `..`, `.`, empty or
   absolute part, no symlink or hard link, at most 256 MiB and 20,000
   entries; its `extension.json` names the pack and the manifest's version;
5. installs that directory as a local one (the manifest checks,
   `minEngine` unless `--any-engine`, into `assets/vfx/<pack>/` through a
   staging directory), then removes the download.

A failed step refuses with what failed and leaves nothing in the game or the
temporary directory. Tests: `crates/og-cli/tests/cli/ui.rs`
(`vfx_add_downloads_a_signed_pack_and_refuses_anything_else`, a release of
`tests/fixtures/vfx-release` served over HTTP), `scripts/release-dist.test.ts`,
and the install test's step 10 (tests/install/run.ts).
