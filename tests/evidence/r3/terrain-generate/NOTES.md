# r3/terrain-generate: `type terrain` makes its field

Branch `r3/terrain-generate` from `origin/main` at 5cc1275d. Brief: the
`terrain-generate` row of docs/phase-1.md, "Plan 2026-10-24, round 3".

## What landed

| Commit | What |
|---|---|
| 62368fa5 | og-cook: the `generate` block (crates/og-cook/src/cook/terrain_generate.rs, its grammar in terrain.rs's header): fractal gradient noise with a domain warp, `mask island <falloff>` or `mask ridge <strength>`, hydraulic erosion by seeded drops (own PCG32), thermal erosion, terraces, `sea` with a `shore` band, rivers carved along the routed flow (priority flood with ε to the sea and the edge, D8 steepest descent, accumulation in samples), `moisture` and `biome` rules writing the control map. Refused beside `heightmap` (both lines named), biomes refused beside `control` and past the terrain's layers. Output is the same tile pyramid; the navmesh bake (e6.rs) reads the field through the same `terrain::field`. TerrainMeta v4 (`generated`, `sea`, `sea_level`, `river_length`); `CookedTerrain.sea_level` |
| 91697d0a | og-runtime, api.ogt: `TerrainInfo.seaLevel` (`Option.f32`, null without `sea`), `og api gen`, `og api gen --wiki`, `og api check` (the wiki diffs are line numbers below the insertion) |
| 5aa9613b | `og cook --report`: a generated terrain's `terrain.generated { histogram (16 bins over the height range), landFraction, seaLevel, riverLength, maps }`; its height (16-bit grey) and control maps as PNGs under the report's directory (`og-out/terrain-maps/`), refused under `assets/` |
| fcdff693 | og-game skill §11: every line of the grammar with its default, an island example, biomes against layers, the report's maps, the three lines placing an og.Water at `seaLevel`; design.md's terrain line and report line |
| 64d20b14 | og-tests: the `terrain-generated` gfx scene (native only, cooked into a project of its own like vox-keyframes) and its test |
| 4aabec93 | fx-showcase: the `islands` station (assets/terrain/islands.ogt, src/islands.ts, tests/islands.og.ts) |
| (last) | this file and the frame read |

## Determinism

Every generator step uses add, subtract, multiply, divide, sqrt, floor,
min, max, comparisons, `next_up`/`next_down`, integer hashes and the
generator's PCG32: no `sin`, `exp`, `pow`, `powi`, `atan` or other libm
call (the biome slope's arctangent and the talus angle's tangent are
series of exact operations, checked against the library to 1e-4° and
1e-5 relative), no hash-map iteration, no threads, sorts with total keys
(f32 bits ordered, index tie-break). Heights are clamped to `height` at
the end so the range means what it says.

`a_generated_field_hashes_the_same_everywhere`
(crates/og-cook/src/cook/terrain_generate.rs) holds the FNV-1a hash of a
256 × 256 field with every stage on (warp, island mask, 20,000 drops,
thermal, terraces, sea and shore, rivers, four biomes): heights' bits,
the control map and the river length, `0x3545_dee3_68d7_5755`. **Run in
the orb (Linux x86_64, dev profile): passes. The coordinator confirms it
on the Mac (aarch64)**: a different value there is a determinism bug, not
a stale constant. The value moved twice while I worked (a shore curve
and the final clamp), so the test does catch a change to any stage.

## Tests and how each fails on the old code

On 5cc1275d the cook refuses the block outright; checked by checking out
the old og-cook, og-cli and og-runtime sources, building `og` and cooking
the skill's two snippets: `line 6: 'generate' is not a terrain entry:
heightmap, height, size, ...`. The new tests do not compile there
(`CookedTerrain::sea_level`, `stats::write_terrain_maps`, the module).

- og-cook unit tests (terrain_generate.rs): the hash above;
  `the_sea_level_flattens_the_shore` (within a third of the band the
  slope is under 0.7 of the land's: 4t − 3t² of it; every band sample
  nearer the sea; outside the band untouched; nothing crosses the
  level: a first version with d·|d|/band failed this, its mean slope over
  the band is the land's); `a_river_reaches_the_sea` (from every river
  sample, down the routed flow, strictly falling, staying in the river,
  until below the sea; the longest runs > 20 samples; > 500 m of rivers;
  most river samples carved > 0.5 m below the dry field);
  `a_biome_writes_its_layer_where_its_rules_hold` (sand full below 11 m
  and empty above 13 m, grass full on 13–88 m flats under 26° and empty
  past 34° or below 9 m, the moisture biome's channel at its weight 0.8
  at most and reached); `refusals_name_the_line`;
  `two_seeds_two_terrains_one_seed_one` (and the plain field spans the
  height range exactly).
- og-cook tests/terrain.rs: `a_generated_terrain_cooks_into_the_same_tiles`
  (two cooks byte-identical, the cook note, 129 × 129 in one tile, range
  −20..≤60, sea level 4 in the blob, the cooked control map land/sea by
  height, the report's histogram summing to the samples, land fraction
  equal to the field's share above the sea, river length, the two PNGs
  written and named, refused under assets/); `refusals_name_the_fix`
  gains generate beside heightmap, biomes beside control, a biome past
  the layers, an unknown rivers key; the heightmap terrain's
  `sea_level` is `None`.
- og-runtime tests/drop_shadows.rs: og.terrain.load answers `seaLevel`
  1.5 for a generated isle and null for the heightmap ramp.
- og-tests tests/terrain.rs: `a_generated_terrain_draws_its_sea_shore_and_biomes`
  (frame read, then the reference).
- fx-showcase tests/islands.og.ts (not run here: `og test` is the
  coordinator's).

Run here, one crate at a time: `cargo nextest run -p og-cook` (161 pass),
`cargo test -p og-runtime --test drop_shadows`, `cargo test -p og-tests
--test terrain a_generated` (the frame checks pass; it then stops at the
missing reference, as intended), clippy `-D warnings` on og-cook, og-cli,
og-runtime and og-tests, `cargo fmt`, the knowledge package's
`bun scripts/validate.ts`.

## Frames read (lavapipe, llvmpipe LLVM 15.0.6)

**terrain-generated** (`og-gfx terrain-generated`, 1280×720, vulkan-cpu):
`terrain-generated.vulkan-cpu.png` here. A 1 km island centred in the
frame from 450 m out to sea at the south-east: open sea (mean RGB
115.8 147.2 166.3 over the bottom corners) to a hazy horizon at y ≈ 185
(sky 213 219 227); the island's south and east faces rock-grey, eroded
into gullies running down to the shore; meadows (29,436 green pixels in
the island's box) on the gentler western and northern slopes and the
plateau; a snow cap on the highest ridge at the top right (747 pixels
over 215 in every channel); a pale beach ring at the waterline (1,485
sand pixels); no visible gap between the island and the water. Two
renders are byte-identical. Draws 15, 24,576 terrain triangles, three
shadow passes.

One defect seen, **not caused by this branch's code paths as far as I
can tell**: 2 black pixels at (430–431, 554) on the underwater shelf
south-west of the island, still there without the water and with a
magenta background (so not a crack), with `heightBlend` 0 or 0.4; the
island scenes on main (terrain-far) have none. It reads like a NaN in
og-render's terrain shading for some normal or texel on lavapipe. The
coordinator should look at the Metal frame for it before recording;
I did not touch og-render.

**islands** (fx-showcase station, run through `og run --commands` with
`OG_NATIVE_ADDON` at the dev addon, in a scratch project holding the
station's islands.ts and islands.ogt; the whole fx-showcase did not get
past its first station in minutes on lavapipe). The headless dump
(`--no-render`) reads `Gen { seaLevel 2, centre 26.45, corner -12 }`: og.terrain.load
answered the generated sea level, og.terrain.sample finds the island 24 m
above the sea at its centre and the corner on the sea floor at the
height range's low. **No frame at the station's camera was read**: at
1280×720 the run did not reach tick 18 in 25 minutes, at 640×360 not in
17 minutes, at 320×180 not reach tick 6 in 10 minutes. One run with the
camera moved closer (x+180, z+200) reached tick 6 at 320×180 in about
30 s, and the valley station's terrain draws at about 1.5 s a tick at
320×180 in the same project. Through all of it one llvmpipe raster
thread does the work while the other 15 idle, so a single screen tile
costs that much. og-gfx draws the generated island (`terrain-generated`,
same generator, renderer path without og-runtime) in 1.7 s. I take this
to be in og-runtime's or og-render's terrain path on lavapipe (perhaps
the same thing as the two black pixels above), not in the cook, and did
not chase it: it is outside this brief. **For the coordinator**: read the
station on Metal before recording; if Metal is also slow there, the
islands camera or the sea's size (six times the terrain's) is the place
to start.

## Metal references owed

- og-tests `terrain-generated` (metal; vulkan-cpu set aside).
- fx-showcase `islands.png` (metal, webgpu, webgl2). No other station's
  frame moves: the station is appended after the light stations, so no
  index or HUD label changes.

## Shared files touched

crates/og-cook/src/cook/terrain.rs (grammar header, `TerrainDoc`,
`field`), crates/og-cook/src/cook/e6.rs (navmesh bake reads `field`),
crates/og-cook/src/format.rs (TerrainMeta v4: **every cooked terrain
recooks** once, the type hash moves), crates/og-cook/src/runtime.rs,
crates/og-cook/src/cook/stats.rs, crates/og-cli/src/cook.rs,
crates/og-api/api.ogt and its generated outputs
(packages/engine/types/index.d.ts, packages/engine/schema/api.schema.json,
packages/knowledge/wiki/*.md), crates/og-runtime/src/terrain.rs (one
line), packages/knowledge/skills/og-game/SKILL.md §11, docs/design.md,
crates/og-tests/src/{scenes.rs,terrain_scenes.rs} (the island's
afternoon light factored into `afternoon()`, island_scene unchanged),
templates/fx-showcase/src/main.ts.

## Left out, or for the coordinator to decide

- The generator is single-threaded and dev-profile og-cook is opt-level
  0: 257² with 60,000 drops cooks in about 0.5 s in the orb, fine; a
  4097² field with a million drops would take minutes. Not optimised.
- `samples` is a `generate` line (the heightmap's pixel count): the
  brief's grammar named no line for it.
- `mask` is one of island or ridge (as the brief has it), not both.
- Rivers are carved channels with sand beds by a biome; there is no
  river water surface (an og.Water per river would need a path mesh).
- The skill's existing heightmap example writes `layer { ... }` on one
  line, which the .ogt parser refuses (`'{' must end its line`); my new
  examples are multi-line (both cooked in a scratch project). The old
  one-liners are left as they were: a fix for the coordinator.
- `OG_GATE=1 sh packages/engine/scripts/build.sh` copied the dev addon
  and og into packages/engine/native and then started a `--profile wasm`
  build (no gate wasm in the orb); I killed it there. `og run` needs
  `OG_NATIVE_ADDON` to name a `.node` file (bun refuses the `.so` in
  target/debug), so I pointed it at packages/engine/native's copy.
