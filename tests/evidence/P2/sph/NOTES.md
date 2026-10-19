# Phase 2, 3D SPH round: evidence (owed row 29)

Branch `p2-sph`, 2026-10-19, Apple M4 Pro (shared with other agents'
builds and gates; load average 27 to 98).

| File | What it shows |
|---|---|
| `fluid-3d-before-after.png` | og-tests' fluid-3d scene on metal: the basin without the fluid (before), with it (after: the committed reference, matched at RMSE 0), and with `smoothing` 0, the spheres the surface is built from |
| `fluid-3d-metal-vulkan-cpu.png` | the same scene's references on metal (left) and lavapipe (right) |
| `fluids3d-station-four-tiers.png` | fx-showcase's station 23, fluids3d, at tick 330 on metal and vulkan-cpu (top), webgpu and webgl2 (bottom), from the committed references |
| `costs.md` | the simulation's CPU per tick (native and wasm32, 500 to 8,000 particles, one and two substeps, a terrain bowl, a group, an emitter into a sink) and the surface's GPU cost on metal and lavapipe |
| `break-and-restore.md` | every new test broken once and seen to fail, then restored |

## The choice: double density relaxation, not PBF

og.Fluid3D is the 2D fluid's model (Clavet, Beaudoin and Poulin 2005) in
3D: one neighbour search and two passes a substep, against position-based
fluids' several constraint iterations over the same neighbours. It is stable at 60 Hz with two
substeps, and groups, layering, mixing, emitters and sinks carry over from
2D with the same semantics. PBF's incompressibility would matter for deep
columns of water; the engine's scenes are pools, pours and splashes of a
few thousand particles, where the relaxation's slight compressibility does
not show and its cost is about half. The reasoning is in
`crates/og-physics/README.md` §Phase 2: 3D particle fluids.

## The surface: screen space, not marching cubes

Sphere imposters into a depth and a thickness target, a separable
bilateral smoothing, and a composite that refracts, absorbs, lights and
reflects (`crates/og-render/README.md` §Phase 2: 3D fluids). It runs the
same on all four tiers (no storage buffer, no compute), and its cost
follows the pixels the fluid covers, not the particles (about 1.1 ms on
the M4 Pro at a third of a 720p frame). Marching cubes was neither built
nor measured. The argument against it is reasoning only: it needs a
density grid filled from the particles and a mesh extracted every frame,
on the CPU for WebGL2.

## Determinism

`tools/wasm-determinism/run.sh` (a513e20): the beyond-rigid-bodies
scenario pours two 3D fluids of one group into a terrain bowl, the heavy
one fed by an emitter, the light one drained by a sink. Host aarch64 debug
and release and wasm32 scalar and simd128 agree: physics_hash
8301574762329360262, the particles and mixes identical across a restore.

## Tiers against metal

The rule: a channel off by more than 32/255 is an outlier; a frame fails
above 0.5 % outliers or RMSE 0.01.

| Frame | vulkan-cpu | webgpu | webgl2 |
|---|---|---|---|
| og-tests fluid-3d | 0.00129 (1 outlier) | (no browser harness) | |
| fx-showcase fluids3d, tick 330 | 0.00146 (2 outliers) | 0.00839 (0.152 %) | 0.00840 (0.133 %) |

The browser tiers sit further from metal than the native ones: the
emitter's pour and the surface's thin edges move slightly between builds,
but every frame stays inside the rule.

How they were made:

- `metal`: og-tests' reference with `OG_GFX_CAPTURE=fluid-3d`;
  fx-showcase's with `OG_FX_CAPTURE=fluids3d.png og test
  templates/fx-showcase`.
- `vulkan-cpu`: the og-lavapipe Docker image (linux/amd64, llvmpipe)
  running og, og-gfx, the addon and the fluids3d test binary cross-built
  with `cargo zigbuild --release --target x86_64-unknown-linux-gnu.2.31`.
  A second run reproduces both frames at RMSE 0. There
  `tests/fluids3d.rs` passes 6 of 6 and all 19 fx-showcase suites pass, fluids3d's
  12 checks included. The other og-tests scenes keep their earlier rounds'
  small lavapipe drift (showcase 0.00338, terrain-near 0.0061, particles
  0.00313, gltf-reference 0.00206; the rest at RMSE 0), all inside the
  rule and not re-recorded.
- `webgpu`, `webgl2`: `bun tests/web/reference.ts [--webgl] --record
  --script "0.5:station:fluids3d" --shots fluids3d@330` from
  packages/engine, in Chrome for Testing with the gate's publish wasm.
  Afterwards `scripts/check.sh web-reference` passed (168 s): 18 of 18
  stations on both tiers, fluids3d at RMSE 0 against its own references.
