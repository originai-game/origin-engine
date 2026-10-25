# Debt round 3 `probe-faces-2`: evidence

Branch `debt/probe-faces-2`, cut from origin/main 58f5609e. It pays the
og-render debt recorded in:
- `tests/evidence/r3/probe-faces/NOTES.md` lines 319-345.
- `docs/phase-1.md` line 6450.
- `tests/evidence/debt2/render-views-2/NOTES.md` lines 264-266.
- `tests/evidence/r4/render-billboards/NOTES.md` lines 97-104.

Every check ran in an orb on lavapipe (`vulkan-cpu`, debug build).
Following AGENTS.md, nothing here ran `scripts/check.sh`, `og test`,
web-reference, a wasm build or a release build. The loop was:
- `cargo check -p <crate>`;
- `cargo clippy --all-targets -- -D warnings` on og-render, og-runtime,
  og-tests and og-api;
- `cargo fmt --check`;
- the unit and integration tests of the crates edited, one binary at a
  time.

To check that each new test bites, its crate's change was stashed and the
test rerun on the old code. The numbers are below.

## What landed, one commit per item

| # | Item | Commit | Test (fails on the old code) |
|---|------|--------|------------------------------|
| 1 | Probe faces loop every local light in terrain, scatter, 3D fluid and full custom shaders | bd041093 | og-tests probes `local_lights_reach_terrain_and_fluids_in_a_capture` |
| 2 | A baked probe's FaceTargets are freed after its capture | bd7631bc | og-render unit `face_targets_are_kept_for_due_and_realtime_captures_only` |
| 3 | Water in a face keeps SSR. Transparents stay out of the face copy, for a reason now in the README | d65ad43f | og-tests probes `water_in_a_capture_reflects_the_face_in_screen_space` |
| 4 | The view glow runs the frame's bloom chain | ec38e0e5 | og-tests targets `a_views_glow_spreads_as_the_frames_bloom` |
| 5 | Billboards sort among transparent meshes, and world text faces the camera (IDL `TextDesc.facing`, `FloatingTextDesc.facing`) | 0bbd9443 | og-tests gfx `billboards_sort_among_transparent_meshes`, `world_text_faces_the_camera`; og-runtime `tests/text_facing.rs` |

### 1. Local lights in probe faces

Problem: fluid3d, terrain_lit, scatter and custom_lit read the main
camera's froxel clusters. Inside a face, those clusters index the wrong
cells.

Change:
- New `surface_local_lights(frag, s, p, n, v)` in `area.wgsl`. It walks the
  cluster's lights when `frame.cluster.x > 0` and every light otherwise.
- Faces run with the cluster count at zero.
- All four shaders call it, so the loop is the same one the standard
  material uses.

Results, a point light inside the probe box:
- Meadow terrain, face -y: blue 0.030 → 0.156.
- Fluid, face +x: blue 0.047 → 0.490.
- On the old code, the capture is identical with and without the light.

Limit (stated in the README under Forward+): the lights a face loops are
the frame's kept light records (at most `MAX_LIGHTS`, 256), kept for the
main view. A light whose range misses the main view lights nothing in a
capture either. Keeping lights for the due faces would change which lights
the main view's shadow budget picks on capture frames, so it stays out.

### 2. FaceTargets freed

- `face_target_sizes(&ProbeFrame)` keeps a size only for a probe whose
  capture is due this frame, or for a realtime probe.
- A baked probe drops its targets after its capture (up to 71 MB at 1024²)
  and allocates them again when its next capture is due.
- A scratch test (deleted) re-captured after the free and matched the
  first capture bit for bit.

### 3. Faces: SSR on water, transparents

- Water drawn into a face now keeps its screen-space reflection
  (`look.y` is no longer zeroed). The SSR reads the face's own colour and
  depth. Planar reflection stays off in faces, because it would be a
  seventh camera per face.
- Result: a wall over the water adds blue 0.076 in the face. The old code
  adds 0.000.
- Transparents stay out of the face copy. The main pass copies opaque
  colour before its transparent meshes draw, and a face follows the same
  rule so that a reflection agrees with the frame it reflects. Copying
  after the transparents would refract glass through glass, and it would
  make the reflection of a pane differ from the pane. The README section
  "Reflection probe faces" says this precisely, and design.md §2.6 and the
  og-game skill match it.

### 4. View glow = bloom

- `shaders/bloom_chain.wgsl` (`downsample13`, `bloom_tent`) is now shared
  by `post.wgsl` and `views_glow.wgsl`.
- A view runs the same mip chain as the frame's bloom: per-level views and
  bind groups in `views.rs`.
- The weight is the bloom intensity when lit bloom is on, and the glow
  intensity otherwise.
- Result: halo profiles at 60-200 px are equal, with bloom and without.
  At 140 px the halo is 188/188; the old code gave 171 against 188.
- The main `bloom` frame is byte-identical before and after (og-gfx
  `--out` and `cmp`).

### 5. Billboards and world text

- A faced sprite without `sortY` (and tone mapped) leaves the sprite
  layer runs and sorts among the transparent meshes by its anchor's view
  depth: `DrawList::before`, `sort_transparent` and `draw_blended`. This
  holds in the main pass, every view and every probe face. Face sprite
  lists are now built per face and culled to it.
- A red billboard behind a blue glass pane: 255/63/63 before, 196/103/210
  now. One in front of the pane is unchanged.
- World text: `RenderScene::text_facing` (`TextFacing { text, facing,
  align }`) anchors every glyph at the text's aligned point, so the text
  turns as one panel.
- The IDL gained `TextDesc.facing` and `FloatingTextDesc.facing` (zero
  value `none`). The `og.sprite` state block writes facing in a trailing
  section only when a text faces, so blocks without one are byte for byte
  the old ones. `tests/text_facing.rs` snapshots and restores it; its
  assertion was checked to bite.
- `og api gen` was rerun, and `og api check` is clean (.d.ts, schema,
  wiki).

## Scripted `og run --commands` frames (lavapipe, 960×540, read)

1. `og run templates/fx-showcase --headless --commands
   "0:station:reflections;40:screenshot" --size 960x540`.
   The room is lit by its two lamps. The chrome and gold spheres reflect
   the red back wall, the blue left wall, the green block and the
   emissive panel from the baked probe, and the wet floor carries the SSR.
   The capture still reads correctly after its face targets are freed:
   it was made once, freed, and is sampled 2,400 ticks later. Hash
   7c5ea21586b67a9b.
2. `... "0:station:views;40:screenshot"`.
   The monitor shows the security camera's view (with the magenta ghost
   on layer 1), the mirror reflects the walker and the ball, and the
   split-screen view sits bottom right. Nothing moved apart from the
   views' glow chain, and this station has no emissive strong enough to
   glow. Hash e0208db471bcd200.
3. A scratch project (copied from templates/game, not committed). It has
   a blue transparent pane (opacity 0.55) across x -4..4 at z 0, a red
   `facing: "camera"` billboard at z -6 behind it, one at z 3 in front of
   it, and the texts "FACING" (`facing: "camera"`) and "FLAT" (no
   facing), seen from (6, 2.5, 8). Command: `"10:screenshot"`.
   - The far billboard is drawn through the glass, tinted pink-violet.
   - The near one is pure red over the glass.
   - "FACING" reads square to the camera, and "FLAT" is foreshortened
     along the world's xy plane.
   - Before this branch, the far billboard drew over the pane in pure red.

## Not verified in the orb

- Metal references: the committed references are metal and the orb has
  only lavapipe. No reference frame was re-recorded, because every new
  test compares two renders of its own.
- The wasm host and the browser tiers. The shaders changed (`area.wgsl`,
  `bloom_chain.wgsl`, `post.wgsl`, `views_glow.wgsl`, `water.wgsl`,
  terrain, scatter, fluid3d, custom_lit), and none of them was compiled
  by naga for WebGPU or WebGL2 in a browser here. The og-render `shaders`
  test, which validates every module through naga, passes (5/5).
- Pre-existing lavapipe failures, with identical numbers on 58f5609e:
  `sprites-array` (RMSE 0.01255), and terrain `water-underwater` against
  metal (RMSE 0.02253, no vulkan-cpu reference).

## Frames the Mac lane must check, and re-record on metal if they moved

- `sprites-billboard`. Its metal reference was already owed. Faced
  sprites now draw in the transparent pass, and this scene has no glass,
  so the change is expected to be nil or sub-threshold. Its vulkan-cpu
  reference passes at RMSE 0.
- Views with glow (`views-*` in og-tests `targets` and gfx). The glow now
  spreads as the bloom does.
- Probe references (`probe-two-rooms`, `probe-outdoor`). Their faces now
  light terrain, scatter and fluid with local lights, and water keeps SSR.
- fx-showcase playtest frames for stations with these features:
  reflections, views, terrain/islands, fluids3d, water, and any billboard
  near glass.
- The browser tiers (web-reference), because shaders changed.

Commands on the Mac, at the branch head:

```sh
scripts/check.sh --only test        # og-tests gfx references (metal)
OG_GFX_CAPTURE=<scene> cargo test -p og-tests --test gfx -- <scene>   # re-record a moved frame
scripts/check.sh --only bun-test    # templates' playtest frames
scripts/check.sh web-reference      # browser tiers
```
