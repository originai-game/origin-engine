# Depth S7 render-light: evidence

Branch `depth/render-light`. Every frame is from lavapipe (llvmpipe, LLVM 15,
the `vulkan-cpu` tier) in an orb. Region means are RGB 0..255 over
`x0,y0,x1,y1`. No Metal or WebGL2 frames were made here; the coordinator
records those references (listed at the end).

## Bake time

`og bake lightmaps templates/fx-showcase --samples 512 --texels-per-metre 32`
bakes `scenes/cornell.ogt`, which has one static model (7 parts), its pages
and one LightVolume (8 × 8 × 8 probes, 0.3 m apart). The run took 1.4 s,
the time written in `cornell.lightmap.ogt`; with the cook it took 1.6 s
wall time. At 256 samples and 24 texels per metre it took 0.7 s wall and
6.3 s CPU. The bake is repeatable at a fixed sample count: same seed, same
pages.

The og-tests scene `baked-cornell` bakes its own box as the scene is built
(64 samples, one thread). This adds about half a second to the 1.7 s it
takes to set up and draw the first frame.

## fx-showcase stations (`og run --headless --commands`)

### `baked`, 640 × 360

Files: `baked.png` (station shown at 0.3 s, captured at 1.5 s) and
`baked-off.png` (after `baked:off`).

With baking on:

- The tall block's side facing the red wall is red, because the bounce
  from the wall lands on it.
- The short block's right face is green, from the green wall.
- The ceiling is grey. The spot under the ceiling points down, so nothing
  but bounced light reaches it.
- The back wall is warm white.

With `baked:off`:

- The same surfaces take the sky's blue-grey ambient light.
- The ceiling is near black.
- No colour bleeds from the walls.

| Region | Baked on | Baked off |
|---|---|---|
| Tall block's red-facing side `236,170,244,290` | 128,80,82 | 94,102,119 |
| Ceiling `240,8,400,20` | 128,127,125 | 48,44,42 |
| Back wall `250,170,310,290` | 164,161,161 | 135,137,145 |
| Red wall `160,100,200,220` | 159,40,33 | 145,37,33 |
| Green wall `450,100,490,220` | 75,140,52 | 63,127,49 |

`baked-1280.png` is the playtest's own frame at 1280 × 720:

- Block side `472,340,488,580`: 128,80,82.
- Ceiling luma: 135.

### `volumetric`, 640 × 360

Files: `volumetric-0852.png` (tick 90, 08:52) and `volumetric-0952.png`
(tick 330, 09:52). The og.DayNight clock advances an hour every four
seconds.

In both frames:

- Fog fills the room.
- The sun throws pale shafts through the window's four panes, slanting
  down to a lit patch on the floor where the bars' shadows split it into
  panes.
- The lattice-cookie spot casts a grid of warm cells and dark bars on the
  floor in front of the camera.
- The lantern's cube cookie dots the left wall with warm round spots.
- The rect light panel under the ceiling is a cyan-white slab that lights
  the wall below it in a soft wash.

Between the two frames, the floor patch moves from under the window
towards the wall and shrinks as the sun climbs:

- Floor at `330,270,470,300`: 180,174,172 at 08:52, 130,127,133 at 09:52.
- Shaft above the floor `320,180,420,230`: 108,111,118 at 08:52, 90,94,104
  at 09:52.

`volumetric-1280.png` is the playtest's frame:

- Sun patch luma 174 against the shaded floor's 84.
- The lattice pool's luma runs from 119 (10th percentile) to 151 (90th).
- Mean change against the frame half a second earlier is 10.0 over the
  floor and shafts.

The playtests `tests/baked.og.ts` and `tests/volumetric.og.ts` assert
these effects with margins. I did not run them through `og test`: their
checks were run as a scratch port against these frames, and every check
passed.

## og-tests gfx scenes (`og-gfx <scene>`, `frames/`)

- **`rect-lights.png`:** three rect lights at night over a glossy floor
  in front of a matte wall.
  - The warm panel facing down throws a long rectangular highlight on the
    boards `300,600,500,660`: 134,117,95.
  - The cyan two-sided pillar lights both the wall behind it and the floor
    in front of it `740,450,800,600`: 153,196,205.
  - The magenta panel facing left lights the floor to its left and
    reflects as a rectangle `1020,480,1100,600`: 211,143,204. Nothing is
    lit to its right.
  - The grey ball between the lights is lit warm on one side and cyan on
    the other.
- **`light-cookies.png`:** a spot through a stained-glass cookie throws
  four panes in dark leading on the floor:
  - Red `200,330,480,400`: 103,22,23.
  - Blue `200,480,400,580`: 43,89,196.
  - White `400,490,680,580`: 161,168,165.
  - Amber.

  A point light with a cube cookie dots the back wall, side wall and
  floor with green round holes (`700,180,760,250`: 55,186,86), and the
  dark between them stays dark (`600,600,700,650`: 15,29,36). The `+Y`
  face is closed, so nothing lands on the ceiling.
- **`volumetric-fog.png`:** a wall with three slots in front of a low sun,
  in fog.
  - Each slot is a bright shaft (`470,200,500,500`: 220,217,211) slanting
    down to a bright bar on the ground.
  - Fog in front of the piers is lit grey by the sun scattering around
    them (`300,200,420,500`: 128,123,116).
  - Behind the middle pier, where no shaft reaches, it stays dark
    (`640,200,720,500`: 16,18,22).
  - The orange spot on the right draws its cone in the fog and its pool on
    the ground (`900,580,1200,660`: 147,119,97).
  - Vertical streaks in the fog are the froxel grid's 160 columns
    resolving the 0.5 m slots.
  - Control runs: without fog the frame is the plain lit wall. With the
    sun's shadow off, the fog is uniformly bright, with no shafts.
  - A debug render tinting each cascade showed the cascades do shadow the
    froxels.
- **`baked-cornell.png`:** the Cornell box baked in-process.
  - The tall block's side facing the red wall `472,340,488,580`:
    102,44,43.
  - The ceiling, lit by bounces only, `480,16,800,40`: 115,109,98.
  - The ball inside is lit by the probes.

  `the_bake_bleeds_colour_and_lights_the_ceiling`
  (`crates/og-tests/tests/light.rs`) builds the same box unbaked and
  asserts that the side is red only when baked and that the ceiling is
  more than 40 luma brighter when baked. It passes on lavapipe.

## Shared files touched

These files are also in the coordinator's merge path with S6 and S8.

- `crates/og-api/api.ogt`:
  - New fields: PointLight.cookie, SpotLight.cookie,
    RenderSettings.bakedLighting, RenderCapabilities.volumetricFog,
    Model.static, and Fog.volumetric, scattering, anisotropy and
    volumetricDistance.
  - Appended block: the VolumetricFog enum, RectLight, LightVolume,
    BakedScene and render.bakedLighting.
  - Regenerated: `index.d.ts`, `api.schema.json`, `bindings.json`,
    `abi.ts`, the hosts' `glue.rs` and the wiki.
  - `packages/engine/src/index.ts` (column factories) and
    `src/light.ts`.
- **og-render:**
  - `lib.rs`:
    - Module declarations.
    - Frame bindings 10 (probes) and 11 (LTC table).
    - Local shadow array layers past `shadows::capacity` for cookies.
    - The apply pass at the end of the Main arm.
    - The lightmap pipelines at group 3.
  - `frame.rs`: rect light records in `local_lights`.
  - `post.rs` (the frame graph): `FramePass::Volumetric`, named
    `volumetric`, which reads `cascades` and `local-shadows`, writes
    `froxels`, and is read by Main. `FULLSCREEN_WGSL` is made
    `pub(crate)`.
  - `scene.rs`: `RenderScene.volumetric_fog`, `baked`, `light_cookies`,
    and `LightKind::Rect`.
  - `shaders.rs`: validates `volumetric` and `cookie`.
  - Lit WGSL:
    - `pbr.wgsl`: the lightmap vertex stream.
    - `pbr_fs.wgsl` and `physical.wgsl`: one call each to
      `ambient_irradiance()` in place of the sky's irradiance.
  - New files: `lightmap.rs`, `volumetric.rs`, `area.rs`,
    `shaders/lightmap.wgsl`, `area.wgsl`, `cookie.wgsl`,
    `volumetric*.wgsl`.
- **og-cook:**
  - `cook/mod.rs`, `cook/scene.rs` (`static true` in the Model row),
    `cook/stats.rs` (the lightmap entry in `cook --report`), `format.rs`,
    `ktx2.rs`, `runtime.rs`.
  - New: `lightmap.rs`, `cook/lightmap.rs`.
- **og-cli:** `main.rs` (the `bake` subcommand and usage), `doctor_project.rs`
  (the `unbaked` warning), `Cargo.toml`.
- **og-runtime:** `extract.rs`, `assets.rs`, `atmosphere.rs`, `api.rs`,
  `engine.rs`, `lib.rs`; new: `baked.rs`.
- **Hosts:** `og-host-node` and `og-host-web` `host.rs`
  (`og_render_baked_lighting`).
- **Other:**
  - og-tests `scenes.rs`, `lib.rs`, `Cargo.toml` (og-bake).
  - `Cargo.lock`.
  - `docs/design.md` §2.6.
  - The og-game skill: new §22, with "Where to look things up" now §23.

## References owed (Metal, by the coordinator)

- og-tests: `rect-lights.png`, `light-cookies.png`, `volumetric-fog.png`,
  `baked-cornell.png` (native only).
- fx-showcase: `references/metal/baked.png`, `references/metal/volumetric.png`.
- The webgl2 references of the same, if wanted: `volumetric-fog` there is
  the raymarch, lit by the sun alone.
