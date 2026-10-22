# Windows x64 release candidate (F7)

Started 2026-10-22 on the `studio-win` runner; native validation on
2026-10-23. Checkout `C:\Users\win\src\origin-engine`, branch `windows-x64`.
Main's 0.2.0 version is used for the local candidate; nothing is published
by this branch. The coordinator merges and the owner authorizes publishing.

## Host and toolchain

- Windows x64 10.0.26200, NVIDIA GeForce RTX 5060, DX12.
- Visual Studio 2022 Build Tools 17.14.37502.11, MSVC 14.44.35207,
  Windows SDK 10.0.26100, C++ ATL (`Microsoft.VisualStudio.Component.VC.ATL`).
- Rust 1.99.1, target `x86_64-pc-windows-msvc`; wasm32 and rust-src installed.
- Bun 1.4.2, Node 22.23.4, wasm-bindgen 0.2.129, Binaryen 133,
  sccache 0.14.0, ShellCheck 0.11.0, cargo-nextest 0.9.149 (unpinned).
- Git Bash runs the build/pack scripts. Cargo discovers the MSVC linker;
  no GNU target or `libnode.dll` workaround is used.

## Portability fixes

The release scripts stage `og_host_node.dll` as `origin-engine.node` and
`og.exe` in `@origingame/engine-win32-x64`. The shared package declares the
third optional platform dependency and the version checker holds it equal.
Rust remains inside the engine; games use the unchanged TypeScript API.

Windows deliberately selects the pinned tsgo/Bun checker: npm has no
Windows tsrs package. Doctor reports the fallback, and bad and good types
are tested through `og check`. Canonical Win32 verbatim paths are normalized
before entering TypeScript or the native asset loader. This also fixes
`og test --filter` and installed starter asset loading. Unix-only shell,
symlink and SIGKILL fixtures remain Unix-only. Panic locations normalize
separators. `.gitattributes` preserves LF text and binary HDR bytes, which
otherwise broke generated-file checks and HDR decoding with autocrlf.
Knowledge mounts compare bytes but not unsupported POSIX mode bits on
Windows, so a second mount and `--check` are now idempotent.

DX12 is enabled and selected automatically on Windows. FXC crashed while
compiling `stylise-paint`; the engine now embeds wgpu's static DXC compiler
on x64 MSVC, with a paint-frame regression test. There is no separate DXC
DLL to install. `dumpbin /DEPENDENTS` confirms the addon still uses the
Microsoft Visual C++ x64 runtime (MSVCP140/VCRUNTIME140); authors need its
redistributable, not the build tools or Rust.

## Final validation and tarballs

- `cargo fmt --check`, `cargo check --workspace --all-targets`, and
  `cargo clippy --workspace --all-targets -- -D warnings`: pass on 0.2.0.
- `cargo nextest run --workspace --no-fail-fast`: **939 passed, 5 failed,
  12 skipped**, 944 run in 291.359 s. All failures are the five reference
  comparisons below; this is not an all-green workspace suite. Log:
  `target/windows-nextest-020.log`. No tests were filtered to produce it.
- Engine package's three typechecks: pass with the pinned tsgo fallback.
- Knowledge mount tests: 6 passed, 92 assertions, including idempotence
  and detection/repair of changed, missing and stray files.
- `scripts/release-bump.ts --check`: every occurrence is 0.2.0.
- `OG_RELEASE=1 sh packages/engine/scripts/pack.sh`: pass; release MSVC
  native binaries plus all five web wasm variants. After the knowledge-only
  mode fix, its staged script was refreshed and the shared package repacked
  with `bun pm pack --ignore-scripts`; native/wasm bytes were unchanged.
- `bun tests/install/run.ts --from packages/engine/dist/pack --version 0.2.0
  --host-only --keep`: **PASS in 87.1 s**, no engine checkout overrides and
  no renderer-disable switch. The installed 3D starter (two playtests),
  runner archetype and 2D starter pass, the publish export's wasm equals
  the packaged module, and knowledge validate/mount/check passes. Log:
  `target/windows-install-final.log`.
- Direct ShellCheck of build.sh/pack.sh still reports their existing
  SC1091 source-following and SC2086 intentional splitting notes.

Installed doctor lines:

```text
ok       renderer      dx12 (a 64x64 frame headless in 668 ms)
optional tsrs          no tsrs package for this platform; og check uses tsgo 7.0.2 through bun x
```

Tarballs in `packages/engine/dist/pack/` (local candidates, not registry installs):

| File | Bytes | SHA-256 |
|---|---:|---|
| origingame-engine-win32-x64-0.2.0.tgz | 33669423 | 1d2c4b7a797de698c25a8048162dfd381e3ab0b848d47f9683e129de5ca2c8d6 |
| origingame-engine-0.2.0.tgz | 24644842 | e08f432baaf0be2ce67d3d61087d5f53535a4fe3e393a932e5d0cd9682d481c2 |
| origingame-create-game-0.2.0.tgz | 15949008 | 229b8d329029e3ca6506304ffbd973dc86b700f7014769259aa5f6c8f43ba26e |

## DX12 reference comparison

`og-gfx --all --capture .amp/in/artifacts/windows/frames` rendered all
84 scenes with static DXC. `scripts/frames-vs-metal.ts` measured them
against `crates/og-tests/references/metal`; full results are in
[`windows-x64-vs-metal.tsv`](windows-x64-vs-metal.tsv). Six exceed the
default comparison rule (RGB RMSE 0.01, at most 0.5% of pixels over 32/255):

| Scene | RMSE vs Metal | Outliers | Failing nextest test |
|---|---:|---:|---|
| atmosphere-night | 0.01650 | 0.091% | atmosphere::atmosphere_scenes_match_their_references |
| atmosphere-moon | 0.01613 | 0.090% | moon::atmosphere_moon_matches_its_reference |
| sprites-array | 0.02048 | 0.136% | gfx::sprites_array_match_their_reference_in_one_draw_per_array |
| stylise-pixel | 0.01484 | 1.649% | capture sweep only |
| tilemap-lit | 0.03037 | 2.745% | gfx::tilemap_lit_matches_its_reference |
| water-underwater | 0.02975 | 3.883% | terrain::water_phase_2_scenes_match_their_references |

The other 78 frames meet that default rule. Paint now renders and its
regression passes. The inspected paint frame shows the checkerboard floor,
material spheres and transparent cube, not black output or shader failure;
the inspected tilemap shows the brick room, central pillar and platforms,
with warm left lighting and dark central/right regions. These inspections
do not establish pixel equivalence. Differences above are measured, not
classified as intentional backend behavior.

`crates/og-tests/src/gfx.rs` selects
`references/<RenderInfo::backend_tier()>/<scene>.png` per frame, falling back
to Metal when that file is absent, failing if neither exists. On this host
the tier is `dx12`. A DX12 reference set would need reviewed, reproducible
captures on the agreed Windows GPU/driver baseline at the scene's fixed
size/tick/AA. Only differences shown to be intentional should receive their
own frame; matching scenes can continue using Metal. UI and project
playtest references are separate harnesses and require their own review.
No DX12 references or tolerance changes are committed.

### The coordinator's review (2026-10-23, at the merge)

The six DX12 captures were read beside their Metal references
(side-by-side and absolute-difference images, `magick compare`):

| Scene | Max channel diff | Mean | Pixels over 20 % | Where |
|---|---:|---:|---:|---|
| atmosphere-night | 234 | 0.23 | 0.079 % | isolated star pixels of the sky's noise |
| atmosphere-moon | 234 | 0.21 | 0.077 % | the same stars |
| sprites-array | 216 | 0.14 | 0.134 % | the texel edges of the 16 pixel-art clocks (nearest sampling at a half-texel tie) |
| stylise-pixel | 51 | 0.21 | 0 % | the dither pattern, nowhere over 20 % |
| tilemap-lit | 206 | 1.60 | 0.140 % | tile edges across the lit room |
| water-underwater | 121 | 1.31 | 0.063 % | the caustic pattern on the sea floor (procedural noise, float differences) |

Nothing moved: every object, light, shadow and UI element is where the
Metal frame has it; the differences are rasterisation tie rules and
transcendental-function precision between the two backends, the same kind
lavapipe shows against Metal. Verdict: the six are not defects and not
intentional backend behaviour either, so no DX12 reference tier is
recorded and no tolerance moves; the five tests keep comparing DX12
against Metal and keep failing there, reported as such in the device row.
A DX12 tier is recorded only if a scene's content ever differs.

## Remaining release acceptance

Publishing requires the owner's npm two-factor authorization (one-time
password or an authorized publishing credential). Ship Windows at the
version main carries when this branch lands, with the shared package's
Windows optional dependency; do not overwrite an already published shared
version. Registry-based installation on Windows and one Windows platform
runner engine thread remain owed. This host has development tools and the
VC++ runtime installed, so it is not a clean-OS dependency test. Neither
Mac nor cloud validation was run here.
