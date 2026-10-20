# Image-based lighting from cooked environment maps, AgX, exposure adaptation

Branch `r-ibl-post` from main at b62dd65 (Apple M4 Pro, Metal; lavapipe in
the og-lavapipe container; Chrome for Testing for the browser tiers).

## Frames (each read)

| File | What it shows |
|---|---|
| `agx-vs-aces-bloom.png` | top: og-tests' `bloom` (ACES); bottom: `post-agx`, the same scene through AgX: lower contrast, the orange and magenta halos paler (orange halo channel spread 58.0 against 70.9), the cores rolling to white, the night sky lifted from (12,15,26) to (18,24,37) |
| `og-tests-environment-map-metal.png` | `environment-map`: chrome (top) and white (bottom) spheres, roughness 0 to 1, lit only by the 64×32 synthetic studio HDR baked at 64, turned 0.6 rad, drawn behind; the mirror shows the sky, a sharp key highlight and the floor below a crisp horizon, softening sphere by sphere |
| `fx-showcase-environment-metal.png` | the `environment` station: five spheres (mirror, brushed, gold, red lacquer, chalk) on a plinth under the 256×128 studio, AgX, adapting auto exposure, the HUD and og.ui badge over it |

## References

| Reference | metal | vulkan-cpu (vs metal) | webgpu (vs metal) | webgl2 (vs metal) |
|---|---|---|---|---|
| og-tests `post-agx` | new | new, RMSE 0.00128, 0 outliers | judged against metal: RMSE 0.00004 | against metal: RMSE 0.00004 |
| og-tests `environment-map` | new | new, RMSE 0.00097, 0 outliers | against metal: RMSE 0.00001 | against metal: RMSE 0.00000 |
| fx-showcase `environment.png` | new | new, RMSE 0.00113, 0 outliers | new (RMSE 0.0005) | new (RMSE 0.00777, 0.147 % outliers: MSAA in place of TAA) |

No existing reference moved. The sky-lit og-tests scenes (`gltf-reference`,
`sky-only`, `showcase`, `overlay`, `exposure`, `bloom`, ...) match their
metal references at max channel difference 0; `fx-2d`, `sprites-20k` and
`point-spot-lights` keep the max difference 1 they already had at b62dd65
(measured there). All 20 fx-showcase suites pass on metal;
`scripts/check.sh web-reference` passed (427 s, before the browser
references of the station were recorded; reproduced at RMSE 0 after).

## Determinism

- `crates/og-tests/tests/exposure.rs`: two renderers adapting at 1 EV/s up
  and 2 down over 330 ticks after a 4 EV drop draw byte-identical frames
  at ticks 29, 31, 120 and 330; mean luma 15.2 at tick 31, 42.1 at 120,
  settled within 1 % of instant metering (139.5); a `restore_state` to tick
  31 redraws tick 120 byte for byte; a state no longer kept meters at once;
  zero speeds equal `Exposure::auto()` bit for bit.
- og-probe `snapshots_and_sweeps_carry_the_renderers_state`: snapshot,
  restore and every sweep value restore the renderer's state.
- The station's `og run` ends on world hash db857402d8f25a35 on macOS and
  on linux/amd64 lavapipe.

## Cook

fx-showcase's `env/studio.ogt` (256×128 `.hdr`, size 128, 256 samples):
0.52 s in a dev build, 132,343 B blob (511 KiB of RGB9E5 texels, 128 KiB
zlib'd). `crates/og-cook/tests/environment.rs`: the cooked map equals
og-render's `bake` of the same image and settings exactly.

## Wasm size (size gate, brotli -q 11)

| Module | b62dd65 | r-ibl-post | Delta |
|---|---|---|---|
| og_host_web-publish.wasm | 3,205,718 B (11,850,273 raw) | 3,215,104 B (11,882,001 raw) | +9,386 B |
| og_host_web-lanes.wasm | 3,209,121 B | 3,220,041 B | +10,920 B |

The publish module is 76.7 % of the 4 MiB budget. The HDR/EXR decoders and
the prefilter (`og-render` feature `bake`) are native only; the wasm gains
the copy and adapt passes, the AgX shader, the blob loader and the API.

## After merging main (9b7f45c physical materials, then 5dc3aa0 web-host-tools)

- The path-aware gate passed at 3a19da8: 9/9 steps, 899 s.
- Per download (brotli), against main 5dc3aa0's own measurements
  (docs/phase-1.md row 44):

| Download | main 5dc3aa0 | r-ibl-post 3a19da8 | Delta |
|---|---|---|---|
| 3D publish | 2,853,865 B | 2,863,821 B (68.3 %) | +9,956 B |
| 2D publish | 2,688,261 B | 2,697,036 B (64.3 %) | +8,775 B |
| lanes | 3,230,782 B | 3,237,472 B (77.2 %) | +6,690 B |

- Merged frames:
  - og-gfx over all 51 og-tests scenes (main's 49 plus post-agx and
    environment-map): every scene, `materials-physical` included, matches
    metal at max channel difference 0.
  - The exceptions are those whose difference main has too, measured on
    main 9b7f45c with the same numbers: `fx-2d`, `sprites-20k`,
    `point-spot-lights` and `lights-256` at max 1, and the terrain and
    water scenes at RMSE 0.0004–0.0066, all within the rule.
  - All 21 fx-showcase suites pass on metal. `physical` and `environment`
    are at RMSE 0.00000 against their references.
- The split shaders needed nothing: `pbr_fs.wgsl` and `physical.wgsl` read
  the frame group's irradiance and prefiltered cubes, which an environment
  map fills in the sky's place.
- A stale `crates/og-tests/reference-scene/cooked/` (cooked before main's
  `COOKER_VERSION` r3) failed `lanes.test.ts` once ("walker.glb.ogb: holds
  type 3ddf…, expected e792…"). That test cooks the scene only when
  `cooked/.cook.ogt` is missing; `og cook crates/og-tests/reference-scene`
  fixed it.
