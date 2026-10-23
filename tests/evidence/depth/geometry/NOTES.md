# depth S1 `geometry`: evidence

Branch `depth/geometry` from main `02241fbb`. Every frame below is a lavapipe
frame (`vulkan-cpu`, this orb). The Metal references of the gfx tests are
left to the coordinator.

## What landed

| Commit | What |
|---|---|
| `a55167c4` | og-render `geometry/`: three.js primitives (cylinder, cone, capsule, torus, ring, circle, and a tube along a centripetal Catmull-Rom curve with parallel-transport frames), `buffer_geometry` (normals computed as `computeVertexNormals`), `info`, and the binned-SAH BVH (`Bvh::build`, `raycast`), tested against brute force. |
| `e34aae3e` | og-render `lines.rs`, `points.rs`, `debug.rs`, `shaders/lines.wgsl`: screen-space quads with a feathered edge and dashes, in the scene layer (depth tested or not) and the overlay layer (after post, before the UI); material `wireframe` (reflect version 7) drawn as unique edges per mesh revision. |
| `29ec6890` | og-tests gfx scenes and tests `geometry-procedural`, `lines`, `points`, `wireframe`, `debug-overlay` (references owed on Metal) and `a_wireframe_draws_its_edges_not_its_faces`. |
| `17856590` | og-physics `debug_lines()`: rapier's debug render as coloured segments (feature `debug-render`). |
| `ecd7d09d` | IDL: `render` additions and the `debug` module (`publish false`). og-runtime `geometry.rs` implements them: buffer, update, info, the primitives, lines and points (immediate, and og.Lines/og.Points), the raycast, og.debug, the `debug:` command and `RenderOptions.debugView`. Both hosts' trait impls (`host_depth.rs`), the TS runtime (`packages/engine/src/depth.ts`), and `og api gen --wiki`. |
| `adcf9f03` | og-runtime test `render_raycast.rs`: render raycast equals physics raycast. |
| `8dbd8756` | fx-showcase station `geometry` (`src/geometry.ts`) and its playtest `tests/geometry.og.ts`. |
| `7641a0a2` | design.md §2.6 paragraph; og-game skill §18 (the old §18 is now §19); in publish, og.debug.view answers none. |

## Frames read

All frames are in `frames/`.

**gfx scenes** (`target/debug/og-gfx <scene>`):
- `geometry-procedural.png`
  - Left to right: a coral cylinder, a yellow cone, a green capsule, a
    blue torus, a violet ring, and a grey circle sector with a wedge
    missing (thetaLength less than 2π).
  - A gold tube follows its curve from the back right into a hook, with a
    shadow that follows it.
  - In front, a buffer-geometry height field is vertex-coloured from green
    to white, its normals are computed, and it is lit smoothly.
- `lines.png`
  - The orange bar is depth tested: the box hides its middle. The blue
    bar has `depthTest: false` and crosses in front of the box.
  - A dashed mint helix climbs around the box, and a yellow loop
    surrounds it.
  - On the left, a zigzag strip grades from orange to blue through its
    per-point colours. On the right is a fan of pink segments.
  - A white dashed screen line runs across the top.
- `points.png`: a sphere cloud coloured by height, gold at the top and
  blue at the bottom, in squares that face the camera. On the right is a
  grid of fixed-pixel yellow points (no size attenuation).
- `wireframe.png`: from left, a solid coral sphere, then a cyan wireframe
  sphere, a yellow emissive wireframe torus, and a green wireframe height
  field. The wireframes show their triangle edges and no faces, and cast
  no shadow.
- `debug-overlay.png`
  - A green debug box, a yellow sphere of three circles and an orange
    arrow stay crisp over a bloomed sphere.
  - The axes show +x red, +y green and +z blue.
  - The magenta box behind the wall shows through it: the overlay is not
    depth tested.

**The station** (`og run templates/fx-showcase --headless --size 1280x720
--commands "0.5:station:geometry;1.50:screenshot:…;2.00:screenshot:…"`, the
playtest's own script):
- `station-geometry-physics.png`, the physics view the station turns on
  (as `2:debug:physics;3:screenshot` would)
  - The procedural terrain is covered by rapier's orange triangle-mesh
    edges, which is its collider built from the same geometry.
  - The five falling bodies show red box and sphere wires and white
    contact normals where they rest on the terrain.
  - The capsule primitive shows its orange capsule collider.
  - The probe's pink ray drops from a "3.40 m" label to its hit, which is
    marked by a pink sphere and axes.
  - The yellow dashed screen frame and the graded zigzag at the top right
    are drawn over the frame, under the og.ui badge.
- `station-geometry-debug-off.png` (`debug:off`)
  - The overlay is gone: the terrain's vertex colours show, teal in the
    hollows and sand on the ridge.
  - The bodies rest on the terrain, and the gold tube runs under its
    dashed control polygon (og.Lines).
  - The primitives row shows the cylinder, cone, capsule, a mint wireframe
    torus, the double-sided ring and a hexagonal circle.
  - The pink-to-blue point cloud (og.Points) is on the right.
  - The screen lines remain, because they are og.render.lines and not
    debug draws.
- `station-geometry-bounds.png` (`debug:bounds`): a green box surrounds
  every drawn mesh. The terrain's box follows its turned transform, the
  tube's box spans its curve, and the ground plane's box is the line at
  the horizon.
- `game-debug-nav.png` (`templates/game`, `1:debug:nav`): the yard's
  navmesh polygon edges in light blue on the ground, and an agent's path
  in yellow at the bottom left.

**Numbers**
- Playtest checks, computed with the playtest's own helpers on the frames
  above:
  - The probe's raycast gives 1 hit at 3.39 m.
  - 54% of the dashed frame's bottom edge is drawn.
  - The physics view has 49,218 orange pixels over the terrain; with
    `debug:off` it has 0.
  - 157,032 pixels changed by more than 20 in half a second, because the
    terrain is rewritten every tick.
- `render_raycast.rs` uses a 289-vertex, 512-triangle height field turned
  π/4 and moved:
  - Three rays that hit agree with physics within 1e-3 m in distance,
    point and normal. A fourth ray misses both.
  - A ray across the ridge gets two hits, nearest first.
  - After `updateGeometry` raises the field by 1 m, the render and physics
    hits agree again after one step.
  - Mutation check: with the ray left in world space instead of mesh
    space, the test fails (9.67 m against 10.16 m).

## Shared files touched

- `crates/og-api/api.ogt`
  - In `render`: enum `DebugView` before `RenderOptions`, then
    `RenderOptions.debugView` and `StandardMaterialDesc.wireframe`
    (appended fields).
  - At the end of `render`, under a `depth S1` banner: the geometry,
    lines, points and raycast declarations.
  - A new module `debug` at the end of the file.
  - The generated outputs are regenerated and `og api check` passes.
- `crates/og-render/src/lib.rs` (the frame graph)
  - The `GpuMaterial.wireframe` colour.
  - `lines.prepare` and the scene uniform before the ribbon frame.
  - `draw_scene` in the main pass before the sprites.
  - `draw_overlay` after glass and before the UI overlay pass.
  - `lines.warm()`.
  - `FrameStats.line_instances`.
- `crates/og-render/src/instances.rs`: `update(…, skip)` leaves
  wireframe-material instances out of the batches.
- `crates/og-render/src/scene.rs`: `RenderScene.lines`, `points` and
  `css_height`, and `mesh_revision` made public.
- `crates/og-render/src/shaders.rs`: the `lines` module.
- `crates/og-render/src/material.rs`: `wireframe` at reflect version 7.
- og-runtime
  - `extract.rs`: `RenderState.geo`, `DrawLayouts` lines, points, physics
    and engine, clearing scene lines and points, and the call to
    `geometry::extract`. `MeshRow` and `mesh_rows` are now `pub(crate)`.
  - `engine.rs`: the layouts.
  - `probe.rs`: `before_tick` records the raycast's meshes.
  - `e6.rs`: registers the `debug` command.
  - `api.rs`: the `wireframe` setter; `add_geometry` and `uv_repeat` are
    now `pub(crate)`.
- `crates/og-nav/src/system.rs`: `NavSystem::paths()`.
- `crates/og-physics`: `debug_lines.rs`, and the `debug-render` feature in
  `Cargo.toml`.
- Both hosts' `host.rs` (13 `og_render_*` members) and `lib.rs`.
- `packages/engine/src/index.ts`: `render` spreads `depthRender`, `debug`,
  and the `LineMode` column.
- `templates/fx-showcase/src/main.ts`: the station is registered, reset and
  shown, `GEOShow` is added to the systems, and the HUD label and command
  help are updated.
- `crates/og-cli/src/main.rs` is not touched.

## Left out, or different from §1

- **`debug:lod`**: no LOD exists yet, so `DebugView` has none, physics,
  nav and bounds. The stream that adds LOD adds its variant.
- **BVH built at load**: a BVH is built when a geometry is made or updated.
  For glTF and other loaded geometry it is built at the first raycast that
  reaches it and rebuilt when its mesh revision changes. Building every
  glTF primitive's BVH at load would cost load time for games that never
  raycast.
- **The raycast inside a system** reads the meshes as the tick started.
  The probe's `before_tick` records them, which refreshes the transform
  hierarchy once per tick. That cost is owed to the wave-end measurement. A system cannot hand the world's
  transforms to the host, which is the same limit as og.physics.raycast's
  recorded colliders. Outside systems it reads the world live.
- **Skinned models** are tested in their bind pose (documented in the IDL).
- **`layers`**: the field exists with default mask 1. Every mesh is on
  layer 1 until S8 lands.
- **Publish builds**: the `debug` module is `publish false`, so its glue is
  compiled out. On the web, `withPublishRefusals` answers the missing
  members; `og.debug.view` maps the refusal to `none`. Not built here
  because the stream does not run publish or wasm builds.
- **Not run (as briefed)**: the gfx `check` tests need the Metal
  references, and the playtest runs under `og test`. Its checks were
  computed on the same frames, from the same script, with its helpers.
  `scripts/check.sh`, the wasm and release builds and web-reference were
  not run.
