# WebGL2 cold start on Windows: background links, no rect-light code without rect lights

studio-win, 2026-10-28 (NZDT). Branch `webgl2-cold-start` on `origin/main`
`f1c5c0c2` (first measured on `397dbc79`, the 0.6.4 release commit; the
commits between change og check and evidence only). Follows `../0.6.1.md`
§Owed and `../windows-x64-de04f53d/NOTES.md` §4: lit shaders linked in 6.3
to 11.0 s each on ANGLE D3D11, and fx-showcase's WebGL2 start took tens of
seconds.

fx-showcase's WebGL2 first frame, cold: **30.4–31.7 s → 7.0–7.5 s in
Chrome, 30.3–31.2 s → 7.6–7.7 s in Edge**; its five stations driven to
tick 420: 56–58 s → 16–17 s. WebGPU unchanged (1.0–1.4 s). No frame moved
(§Frames).

## Host

- Windows 11 Pro 25H2 (10.0.26200), NVIDIA GeForce RTX 5060, driver
  610.88 (32.0.16.1088, 2026-07-22), 12 logical CPUs.
- Chrome 155.0.8059.41, Edge 155.0.4283.45 (both have 156 staged, not
  running). WebGL2: `ANGLE (NVIDIA, NVIDIA GeForce RTX 5060 (0x00002D05)
  Direct3D11 vs_5_0 ps_5_0, D3D11)`; WebGPU: Dawn on D3D12.
- The desktop Chrome and Edge (the user's profiles) were running
  throughout; every measured browser was a headless one with its own
  fresh `og-cdp-*` profile. No build ran during a measurement.

## Method

The web export of templates/fx-showcase (`webBuild`, as
`tests/web/reference.ts` makes it) from the gate's wasm
(`scripts/check.sh --only wasm-opt`, then `OG_GATE=1 sh
packages/engine/scripts/build.sh`), served as `reference.ts` serves it, on
`reference.page.ts`'s page (no gate, `prime: false`). One fresh headless
browser and profile per run, so Chrome's shader cache is cold. Three runs
per browser and tier. On the page's clock (0 = navigation):

- **first frame**: the shell's first `frame` message (its `boot` phases:
  `attach` makes the renderer, `present` draws the first frame and makes
  its pipelines);
- **warm**: the Worker's `renderer: N pipelines made behind the gate in X
  ms` (with background links it counts pipelines started, not linked);
- **stations**: after the warm line, the five stations' script
  (`reference.ts`'s, ticks to 420) driven one `frame` a call, as
  `reference.ts` does, without the captures; done when tick 420 is drawn.

Per program on WebGL2: a module in front of `game.js` wraps
`WebGL2RenderingContext.prototype` and times each program from its first
`compileShader` to the return of its first `LINK_STATUS` read (`total`),
and the time spent inside the status reads (`blocked`: the engine thread
waiting). The harness is scratch (not committed).

## Before and after

Seconds, runs 1 / 2 / 3. **A** is `397dbc79`; **B** adds the background
links (wgpu-hal patch 2); **C** adds the GL tier's rect-light switch (this
branch).

### WebGL2, Chrome

| | A | B | C |
|---|---|---|---|
| first frame | 31.7 / 30.8 / 30.4 | 22.3 / 22.1 / 22.5 | **7.5 / 7.3 / 7.0** |
| of which `attach` | 8.7 / 8.8 / 8.7 | 0.2 / 0.2 / 0.2 | 0.2 / 0.2 / 0.2 |
| of which `present` | 22.2 / 21.3 / 21.0 | 21.4 / 21.2 / 21.6 | 6.5 / 6.2 / 6.1 |
| warm (40 pipelines) | 14.5 / 15.2 / 14.5 | 0.28 / 0.27 / 0.29 | 0.27 / 0.28 / 0.27 |
| stations done | 57.8 / 57.3 / 56.0 | 41.4 / 40.6 / 42.2 | **16.9 / 16.4 / 16.1** |
| slowest frame (tick: s) | 1: 10.2 / 9.9 / 9.7 | 307: 16.0 / 15.5 / 16.8 | 307: 6.2 / 6.2 / 6.0 |
| engine thread blocked on status | 55.0 / 54.5 / 53.2 | 37.4 / 36.7 / 38.4 | 12.8 / 12.4 / 12.1 |

### WebGL2, Edge

| | A | B | C |
|---|---|---|---|
| first frame | 30.3 / 31.1 / 31.2 | 23.3 / 23.2 / 23.6 | **7.7 / 7.6 / 7.6** |
| of which `attach` | 8.3 / 9.2 / 9.0 | 0.2 / 0.2 / 0.2 | 0.2 / 0.2 / 0.2 |
| of which `present` | 21.3 / 20.9 / 21.1 | 22.2 / 22.2 / 22.6 | 6.7 / 6.4 / 6.5 |
| warm (40 pipelines) | 14.9 / 14.8 / 14.8 | 0.30 / 0.28 / 0.31 | 0.31 / 0.28 / 0.29 |
| stations done | 56.2 / 57.0 / 57.4 | 43.2 / 44.0 / 43.2 | **16.7 / 16.3 / 16.6** |
| slowest frame (tick: s) | 1: 9.7 / 9.7 / 10.1 | 307: 16.7 / 17.7 / 16.3 | 307: 5.9 / 5.7 / 6.0 |
| engine thread blocked on status | 53.6 / 54.0 / 54.3 | 38.8 / 39.8 / 38.9 | 12.6 / 12.1 / 12.6 |

### WebGPU (not touched by either change)

| | A | C |
|---|---|---|
| Chrome first frame | 1.1 / 1.2 / 1.1 | 1.0 / 1.1 / 1.0 |
| Edge first frame | 1.3 / 1.4 / 1.3 | 1.2 / 1.2 / 1.2 |
| Chrome warm (45) / stations | 0.58–0.59 / 6.8–7.1 | 0.57–0.63 / 6.8–7.0 |
| Edge warm (45) / stations | 0.63–0.70 / 7.8–8.0 | 0.62–0.68 / 7.7–7.8 |

A 4.2 to 5.0 s frame at tick 292 in every WebGPU run, A and C alike, is
not investigated here.

## Per program (Chrome run 1; Edge within 0.5 s of it)

**A**: 68 programs read one after another, 54 under 200 ms, their times
adding to 55.0 s, all of it blocked. Over 1 s:

| Program (vertex stage; fragment shader's length) | Made | A |
|---|---|---|
| lit `lit_vertex`, 70,103 chars (the pbr pipeline made with the renderer, `EFFECTS` off) | attach | 8.1 s |
| lit `instance_matrix`, 70,334 | first frame | 10.4 s |
| lit `lit_vertex`, 70,102 | first frame | 10.4 s |
| terrain (`terrain_look`), 61,117 | warm | 7.8 s |
| screen pass (`view_depth, world_at, depth_at, tangent`), 43,141 | warm | 3.6 s |
| lit skinned (`skin_matrix`, `morph_vertex`), 70,102 | station tick 1 | 10.0 s |

(The 70,102 ones differ from the 70,103 one in one character, read as
`EFFECTS` `true` against `false`; the variant names are read from the GLSL,
not logged by the renderer.)

**B**: the three lit programs of the first frame link at once and finish
together, each slower: 17.8, 20.8, 21.1 s (Edge 16.3, 21.4, 22.0 s). The
skinned lit program is made at tick 1 and first drawn at tick 307, where
its link is still running beside the warmed terrain and screen passes.

**C**: the same three in 5.4, 6.4, 6.4 s (Edge 5.6, 6.3, 6.6 s), their
fragment shaders 145 characters longer (the override and its two guards);
the skinned one in 8.5 s (Edge 8.2 s), 6.1 s of it waited for at tick 307.
fx-showcase has no rect light, so every lit pipeline it makes leaves the
rect-light code out.

## Why each change

### ANGLE D3D11 links in parallel, but scales about 1.3×

A raw page (no engine) compiles and links N copies of the pbr lit
program (fx-showcase's GLSL as wgpu writes it, each copy made unique by a
constant in `main`), all started before any is read, then polls
`COMPLETION_STATUS_KHR` every 5 ms; a fresh Chrome per N:

| N | All linked | Per program |
|---|---|---|
| 1 | 8.3 s | 8.3 s |
| 2 | 13.4 s | 6.7 s |
| 4 | 23.6 s | 5.9 s |

So `KHR_parallel_shader_compile` starts the links at once and the engine
thread no longer waits for a program it does not draw with, but two links
take 1.6× one and four 2.8×: about 1.25 to 1.4× the throughput of one
after another on 12 logical CPUs. Background links alone (B) save 25 to
30 %; the lit program's own FXC time is what is left.

### The rect-light code is most of a lit program's link

The same raw page, one program per fresh Chrome, Chrome run twice:

| pbr lit fragment shader | Link |
|---|---|
| as wgpu writes it | 8.5 s, 7.9 s |
| `local_light`'s rect-light branch inside `if (AREA_LIGHTS)` with `const bool AREA_LIGHTS = false` | 3.6 s, 3.5 s |
| that branch deleted | 3.7 s, 3.5 s |
| `area_shade` and `rect_shadow` returning at their top under `!AREA_LIGHTS`, false | 3.6 s, 3.5 s |
| the same, true | 8.1 s |

ANGLE and FXC fold a constant-false branch as well as deleting it: the
LTC evaluation (`ltc_lookup`, `ltc_evaluate` through the 16 KiB constant
buffer) and the rect shadow's 40 taps are about 4.5 s of the 8 s.
The LTC uniform block stays active (block index 7 of 9 either way), so
wgpu's binding of it by name still finds it.

## What this branch changes

- `third_party/wgpu-hal` (patch 2 in its ORIGIN.md): on WebGL2 with
  `KHR_parallel_shader_compile`, a pipeline's compile and link are started
  when it is made, and its link status and uniform locations are read the
  first time a pass sets it. Native GL, Metal, Vulkan and DX12 untouched.
- og-render, the GL tier only: the standard, physical and custom full-lit
  modules read area.wgsl through `area::wgsl(tables)`, which on the
  uniform-table tier declares `override AREA_LIGHTS: bool = true` and
  returns from `area_shade` (no light) and `rect_shadow` (unshadowed) at
  their top when it is false. The renderer's `area_lights` is false on the
  GL tier until a frame's scene has a rect light; until then the lit
  pipelines (standard, lean, array maps, physical, material shaders,
  lightmapped) are made with `AREA_LIGHTS` false, which is exact for those
  frames. The first frame with a rect light turns it on for good and makes
  the lit pipelines again (`forget_lit_pipelines`, shared with the MSAA
  switch), as they were before this change. On the full tier
  `area::wgsl` is area.wgsl unchanged and no pipeline sets the constant:
  Metal, DX12 and WebGPU compile the same modules with the same constants
  as before (`tests/shaders.rs` holds that no full-tier module mentions
  `AREA_LIGHTS`; it also writes the three GL lit modules with
  `AREA_LIGHTS` false as GLSL ES).
- `scripts/check.sh`: the wasm steps' inputs (`rule_wasm`) include
  `third_party`; before, a change only to the vendored wgpu-hal let the
  gate reuse the last wasm. Checked with the gate's own `og_unchanged` on a
  tree that differs from the green one only in wgpu-hal's device.rs: the
  old rule skips, the new one rebuilds.

A scene that gains its first rect light during play on WebGL2 pays the lit
links again then (about 5 to 8 s on ANGLE D3D11, in the background, waited
for by the first frame that draws a lit batch); a scene with one from its
first frame pays them once, as before this change.

## Ruled out, and why

- Polling `COMPLETION_STATUS_KHR` from the engine: wgpu has no "pipeline
  ready" query and a pass that sets a program still linking cannot draw
  without it, so the read waits there either way; the patch reads nothing
  before then. Polling would only keep the Worker answering messages
  during that wait, not shorten it.
- Making more pipelines up front so they overlap: with 1.3× scaling,
  extra lit links slow the ones the first frame needs.
- A separate lighter GL lit shader: the override leaves the same shader
  for frames with rect lights and needs no second copy of the lighting.

## Not done

- Terrain (`terrain.wgsl` reads area.wgsl through `surface_local_lights`,
  7.8 s on A), the scattered copies, the 3D fluid and volumetric fog still
  link with the rect-light code: their pipelines are made once
  (`Deferred`), so following the switch needs them made again. fx-showcase
  warms terrain behind the gate; its link runs beside the skinned lit
  program's and lengthens the tick-307 frame.
- Folding the two `EFFECTS` variants of the pbr lit program into one
  (a GL-tier runtime branch) would leave one lit program fewer in the
  first frame.
- metal and ANGLE Metal on the Mac: the coordinator's.

## Frames

On the build with the switch (label C), Chrome 155.0.8059.41 and Edge
155.0.4283.45:

- og-render: `cargo test -p og-render` (tests/shaders.rs included),
  `clippy -D warnings` and `fmt --check` pass. tests/shaders.rs checks
  that the full tier's WGSL has no `AREA_LIGHTS` and allows the GL tier's
  three "no rect lights" variants.
- `og-gfx --all` on DX12: `dx12-vs-metal.tsv` is byte for byte the one in
  `../windows-x64-de04f53d/` (397dbc79/fa591807); only the known six
  exceed the limits, with the same numbers.
- fx-showcase stations, 5/5 in both browsers, the de04f53d RMSEs:
  WebGL2 post 0.00675, lens 0.00675, fx 0.00469, 2d 0.00290, stylised
  0.00864; WebGPU 0.00455, 0.00676, 0.00392, 0.00288, 0.00863.
- web-reference (`reference-gfx`), every scene in Chrome and Edge on
  WebGL2 and WebGPU: 548 rows, 538 with the same RMSE as de04f53d's
  `web-scenes.tsv` and 10 that are n/a there too (msaa on WebGPU; taa,
  shadows-local-8, sprites-untonemapped and hooks-planar on WebGL2). The
  scenes with rect lights keep their numbers on WebGL2: rect-lights
  0.00113, light-cookies 0.00124. Bloom and post: bloom 0.00119,
  bloom-unreal 0.00127, fx-2d 0.00140, showcase 0.00969 (WebGL2) and
  0.00990 (WebGPU), post-* unchanged. The fails are de04f53d's
  (atmosphere-night, atmosphere-moon, water-underwater, stylise-pixel,
  sprites-array, tilemap-lit, and og-ui/glass 0.01728 on WebGL2, which
  stream A's `webgl2-level-size` fixes; it is not on this branch).
- The WebGL2 lanes got shorter: reference-gfx 160–165 s (about 250 s
  before), the fx-showcase WebGL2 stations 17–18 s.
