# Benchmarks

The measurement pages and scripts from the spike (`docs/spike-results.md`).

- `threejs-compare.ts` — the design §7 comparison of the engine itself with three.js 0.180 in headless Chrome for Testing:
  the complex scene (`complex/` against `three-complex.js`; the judge of the 1.5× budget, below), the §7 reference
  scene (`crates/og-tests/reference-scene` against `three-reference.js`) and a 20,000-sphere per-object field
  (`spheres/` against `three-spheres.js`), engine on WebGPU and WebGL2 (`engine.page.ts`), three on WebGL2
  (and WebGPU for the complex scene and the instanced spheres); `--shots` for screenshots of both sides;
  `BENCH_CHROME_FLAGS` adds Chrome flags (`--use-angle=gl-egl` in an orb without a GPU). Results:
  `tests/evidence/P4/web-host-tools/threejs-comparison.md`.
- `compare.html` — the reference scenes in three.js 0.180 (instanced, `naive=1`, `sphere=1`, `moving=1`).
- `spike-index.html` + `spike-worker.js` — the engine side of the comparison. They loaded the
  spike's wasm (`WebSpike`); they are re-pointed at `og-host-web` once it ships (M2), until then
  they do not run.
- `serve.ts` — a static server with COOP/COEP and a `POST /result` sink (`bun serve.ts`).
- `run-chrome.sh <url> <seconds>` — headless Chrome for Testing with vsync off; pages POST their numbers.
- `views.ts` — the TypeScript typed-view system benchmark (`bun views.ts 10000`).
- `e2b-run.ts` — runs a Linux binary headless in the platform's E2B `probe` sandbox (billable). E2B is the production provider only; the development sandbox is InstaBox since 2026-10-22 (`docs/release.md` §1), checked by `tests/install/instabox.ts`. This script and `crates/og-tests/scripts/capture-vulkan-cpu.ts` stay with the lavapipe measurements they were written for, set aside with the CPU sandbox requirement.

Models for `compare.html` are the ones in `crates/og-tests/models/`; copy or symlink them to `tools/bench/models/`.

## The complex scene and how three.js is matched

`complex/` is the scene that judges docs/design.md §7's 1.5× budget (the
owner's decision of 2026-10-25): WebGPU is the primary tier, WebGL2 is
reported beside it, and the §7 reference scene stays a reported secondary.
It is a level at the size of a shipped Discover game, not a stress field:

| | count |
|---|---|
| ground | one 320 m model, 256 × 256 quads (131,072 triangles), four textured materials (grass, dirt, rock, sand) with a normal map; a lake under a blended water plane |
| props | 5,261 copies of 19 models, 80 mesh parts in 60 materials, 3.03 million triangles: 1,100 pines, 700 oaks, 900 rocks (three shapes), 900 bushes, 300 berry shrubs, 180 palms, 24 islets, 260 crates, 240 barrels, 420 fence panels, 96 lamp posts, 36 houses, 32 market stalls, 40 cars, 6 stairs, 3 dropships, 24 ghost carts |
| moving props | the 24 ghost carts, moved round the ring road every tick |
| transparency | the water, the houses' window glass, the dropships' cockpits, the ghost carts, the smoke |
| skinned characters | 48 on two models: 24 walkers (12 joints, five clips) and 24 villagers (21 joints, nine clips of examples/kestrel-stair's library), each on its own clip, speed and phase |
| lights | the sun with three 2048 cascades over 120 m; two spot and two point lights, all four shadowed (512 maps, cube faces for the points) |
| particles | 20 emitters (4 fires, added; 16 smokes, blended), at most 64 particles each |
| post | AO, bloom, ACES; the tier's antialiasing |

Everything stands where `complex/src/layout.js` puts it, from a seeded hash
and no clock; the engine's `src/main.ts` and `three-complex.js` both read
that file, so the two pages place the same copies, characters, lights,
emitters and camera path. `complex/tools/make-assets.ts` writes the models,
textures and emitters (the engine's CC0 models copied in: walker, palm,
berry, islet, car, ghost, stair; the dropship is MIT, `templates/game`);
`threejs-compare.ts` runs it when it or the layout changed, then cooks.
Two runs of 120 ticks hash the same (`og run tools/bench/complex
--headless --ticks 120`: `814bc0dd60cc490f`, 5,336 entities).

| Feature | Engine | three.js 0.180 | How it is matched |
|---|---|---|---|
| models | `og.load.gltf` of `assets/models/*.glb`, cooked | `GLTFLoader` of the same `.glb` files | the same files |
| textures | cooked KTX2: ETC1S colour, UASTC normals and linear data, transcoded per device | the same cooked KTX2 read out of `cooked/*.ogb`, transcoded by `KTX2Loader` | the same block-compressed texels on both sides; `?ktx2=0` gives three the PNGs instead |
| levels of detail | `RenderSettings.lodBias 0`: every model at its full mesh | none | three has no LOD, so the engine draws as many triangles as three does |
| instancing | automatic, by (geometry, material), culled per instance | `InstancedMesh` per (mesh part, material) and 40 m tile of the ground (878 draws' worth), culled per tile; `&tiles=0` one per part over the whole level | per-tile is the way a three.js author instances a large level so its frustum culling still works; it is three's faster variant here, and per part is reported beside it |
| moving props | a system writes 24 transforms per tick | 24 matrices per part rewritten per frame, bounding sphere recomputed | the same work |
| skinned characters | `og.Skeleton` + `og.AnimationPlayer` | `SkeletonUtils.clone`, a `SkinnedMesh` and an `AnimationMixer` each | the same models, clips, speeds and phases |
| sun shadows | three cascades of 2048 over 120 m | `CSM` (WebGL2) and `CSMShadowNode` (WebGPU): three cascades of 2048, practical split, over 120 m | the same count, size and distance |
| local lights | 2 spot, 2 point, all shadowed, clustered (forward+) | 2 `SpotLight`, 2 `PointLight`, all shadowed (512, cube) | the same lights and shadow maps; three shades every light in every fragment where the engine culls them per cluster, which is the engine's design and is counted as its advantage, as on the reference scene |
| particles | `og.ParticleSystem` of `fx/fire.ogt` and `fx/smoke.ogt` (GPU on WebGPU, CPU on WebGL2) | the same emitters from the same table (`layout.js` `FX`), simulated on the CPU, one instanced billboard draw per blend mode | the same emitters, rates, lifetimes and blend modes; three has no particle system, so its simulation is the simplest one (no noise, no soft particles, which the emitters do not ask for) |
| transparency | blended materials sorted back to front | `transparent` materials sorted back to front | the same materials |
| sky and IBL | `RenderSettings.sky` | `Sky` (WebGL2) or `SkyMesh` (WebGPU) prefiltered by `PMREMGenerator` | as `three-reference.js` |
| AO | GTAO on WebGPU, SSAO on WebGL2 (the tier's) | `GTAOPass` (WebGL2), `GTAONode` (WebGPU) | three runs GTAO on both tiers, the costlier choice on WebGL2 |
| bloom | the engine's | `UnrealBloomPass` (WebGL2), `BloomNode` (WebGPU), strength 0.25 | one pass each |
| tone mapping | ACES | ACES (`OutputPass` or the renderer's) | the same curve |
| antialiasing | WebGPU: TAA; WebGL2: MSAA 4× (the tier's) | WebGL2: SMAA; WebGPU: `TRAANode` | WebGPU: temporal on both sides. WebGL2: three's TAARenderPass re-renders the scene per sample and would not be the same work, so three takes SMAA, a post pass cheaper than the engine's MSAA 4× |
| frame | `frame` call: one 1/60 s tick and one render | one `requestAnimationFrame`, the simulation advanced 1/60 s | as for the other scenes (header of `threejs-compare.ts`) |

Where the two cannot be the same, the choice goes to three: SMAA instead of
MSAA on WebGL2, the per-tile `InstancedMesh` that three's culling needs
(reported beside the per-part one), no levels of detail on the engine side,
and a particle simulation with less work than the engine's. Clustered lights
are the engine's design, not a setting it could turn off; with four local
lights their effect is small.
