# Debt: geometry, timeline, video (S1, S2, S3 "Left out")

Branch `debt/geometry-video`, from `50f1dafd`. Each item is its own
commit, and each comes with a test that fails on the old code. The
debug-overlay text labels (`93c52aaf`) were already merged and were skipped.
Frames were read on lavapipe (`vulkan-cpu`) with `og run --commands`. No
gate, `og test`, web-reference, wasm build or release build was run.

## What landed

1. **BVH in the cook** (`4a65dae7`).
   - Each model primitive's raycast BVH is cooked into the model's `.ogb`.
     There are two zlib sections:
     - `bvh_nodes`: `og.ModelBvhNode` rows, with bounds quantised on the
       primitive's own position grid.
     - `bvh_tris`: the triangle order.
   - `PrimitiveRow` is version 2 and carries the ranges. A type-hash change
     recooks the blob as usual.
   - When a glTF loads (`assets.rs` `add_gltf`, `stream.rs`), the mesh
     adopts the cooked BVH (`geometry.rs` `adopt_cooked_bvh`, og-render
     `Bvh::from_parts`). The lazy build remains only for geometries made or
     updated at run time.
   - `model { raycast false }` opts a model out. `og cook --report` gains
     `bvhBytes`. The og-game skill and design.md §2.6 document the line and
     the cost, about 6.6 B per triangle compressed.
   - Tests:
     - og-runtime `render_raycast::a_cooked_model_raycasts_without_building_a_bvh`
       checks that `Bvh::builds()` is unchanged after the first raycast. It
       fails on the old load, which builds once.
     - og-cook `tests/raycast_bvh.rs`: the cooked BVH equals the one a
       build over the cooked mesh makes, and
       `raycast false` writes none.
2. **Tube along an og.path** (`81ed13f1`).
   - `og.render.tubeGeometry({ path })` takes a loaded `type path` asset.
     og-render `tube_along` samples the path by arc length.
   - Every tube, including one built from points, now spaces its rings by
     arc length, as three.js's `TubeGeometry` does through
     `getSpacedPoints`. Before, rings were spaced by parameter, so existing
     tube frames move.
   - fx-showcase's geometry station draws a mint tube along the new
     `assets/scenes/geo-rail.ogt`, a closed bezier.
   - Tests:
     - og-runtime `tests/tube_path.rs`: a tube from a path equals the tube
       from the path's samples.
     - og-render unit tests.
     - The station playtest counts mint pixels (more than 800).
3. **Timeline hot reload** (`d1576245`).
   - A playing timeline kept its old keys after a reload, for two reasons:
     - `assets::refresh` reread the manifests only when a glTF was loaded.
     - Nothing rebuilt a loaded timeline.
   - Now the manifests are always reread. A loaded timeline stores the hash
     of its output, and `timeline::refresh` (called from probe.rs
     `after_reload`) rebuilds a changed timeline in place. Its time and
     state block are kept.
   - Test: og-runtime `timeline::a_hot_reload_keeps_playing_from_the_same_time_with_the_edited_keys`.
     On the old code the box's y stayed 0.
4. **The intro's audio** (`61c31247`).
   - Bug: music voices set `looping` on og.AudioSource, but the field is
     `loop`. The voice was refused, and `og render --timeline intro --audio`
     stopped at tick 360.
   - Fixed. templates/game gains `assets/audio/sting.ogt` (levelup preset)
     and a music track in `intro.ogt` (6 s to 9.5 s, loop, fades). og-cook
     `template.rs` now counts 23 outputs.
   - Test: packages/engine `og-render.test.ts` "the intro's audio and music
     tracks…" reads the rendered wav and frames.json. It checks:
     - silence before the first cue;
     - the blip, click and music voices where they play;
     - silence after the fade.
     It fails on the old runtime.
5. **Per-player video textures** (`0638b3e5`).
   - `FrameTexture::view` gives a texture of its own over the shared frames
     (an `Rc`). og.video assigns one to each player and writes it to
     `VideoPlayer.texture`.
   - Test: `og-video.test.ts` "two players of one video at different times
     show different frames" (frames 90 and 105). On the old code both had
     texture 4294967296.
6. **Web frame capture** (`c967651a`).
   - With `set_keep_frame`, the renderer copies the presented frame into a
     kept texture before submit (`capture.rs`). For this, the surface gets
     `COPY_SRC` where supported, and the offscreen target `COPY_DST`.
   - On the web host, `Capture::frame` reads that texture back. The
     readback is deferred, because `map_async` resolves on a later frame.
     `og_web_render_frame` is exported (not in publish builds), and
     `worker.ts` answers `renderFrame`.
   - Tests:
     - og-tests `render::a_kept_frame_is_what_a_capture_reads` passes on
       lavapipe.
     - `tests/web/page.ts` checks a captured frame. **Not run** here: there
       is no wasm build in this round.
   - wasm32 clippy is clean for the dev and publish features.
7. **Video on a material map and a UI image** (`2ac32d23`).
   - The video fixture's `surfaces` command shows the player's texture
     twice:
     - as `map` of an unlit og.Mesh plane on the left half;
     - on an og.ui image on the right half.
   - Both rendered correctly on the first run, so there was no runtime bug.
     The debt was that neither path had ever been rendered.
   - Test: `og-video.test.ts` "a video frame shows as an og.Mesh material
     map and on an og.ui image". Each half must have a mean above 60 and be
     nearer the source's frame 90 than frame 30, with RMSE below 0.15. Both
     breakages were checked:
     - a black mesh material fails with a mean of 0;
     - a texture-less UI image fails with RMSE 0.716.

## Frames read (lavapipe)

- `frames/station-geometry-debug-off.png`: the fx-showcase geometry
  station. The mint path tube circles the point cloud with a saddle shadow,
  and the gold tube follows its spline. About 2,060 mint pixels lie in
  x 900–1180, y 200–360.
- `frames/video-pair-vulkan-cpu.png`: the left half shows `03.000 / 90`,
  the right half `03.500 / 105`.
- `frames/video-surfaces-vulkan-cpu.png`: frame 90 twice. Region means
  (RGB):
  - mesh, x 0–640, y 180–540: 108 / 111 / 115;
  - UI image, x 640–1280: 113 / 118 / 121.
  The mesh is slightly darker because it goes through the scene's tone
  mapping and the UI does not.
- Intro wav (`og render --timeline intro --audio`), peak per 0.25 s:

  | Time | Peak | Voice |
  | --- | --- | --- |
  | 0–1 s | 0 | silence |
  | 1 s | 0.30 | blip |
  | 5 s | 0.245 | click |
  | 6–9 s | 0.12–0.18 | music sting |
  | from 9.75 s | 0 | silence |

## `og cook --report` (item 1)

Model blob size in bytes, before → after, and `bvhBytes`:

| model | tris | before | after | growth | bvhBytes |
| --- | ---: | ---: | ---: | ---: | ---: |
| dropship | 16,512 | 171,609 | 280,686 | +64 % | 108,384 |
| trickster | 18,641 | 387,538 | 560,433 | +45 % | 172,765 |
| tree | 3,160 | 27,125 | 53,758 | +98 % | |
| walker | | 3,620 | 4,422 | | |
| cornell | | 2,853 | 3,245 | | |
| puppet2d | | 4,829 | 5,989 | | |
| stout | | 6,150 | 7,865 | | |
| strider | | 6,214 | 7,965 | | |
| window-room | | 2,979 | 3,556 | | |

Project totals:

- templates/game: 2,251,433 → 2,361,312 (+4.9 %).
- fx-showcase: 7,614,190 → 7,820,243 (+2.7 %).

Uncompressed, the BVH took about 17 B per triangle (dropship +161 %),
which is why the sections are zlib-compressed. The choice is the model's
`raycast` line, documented in the skill. There is no size threshold.

## Shared files touched

- crates/og-api/api.ogt and its generated outputs (`host.rs`, `api.schema.json`,
  `types/index.d.ts`, the wiki).
- og-render:
  - `lib.rs` and `post.rs`, both shared with render-light: only the
    keep-frame copy and the `COPY_SRC`/`COPY_DST` usages.
  - `capture.rs`, `geometry/*`, `video.rs`.
- og-runtime:
  - `assets.rs`, `stream.rs`, `probe.rs`, `timeline.rs`, `geometry.rs`,
    `video.rs`, `record.rs`;
  - `extract.rs`, doc only.
- og-probe `hooks.rs`, doc only.
- og-cook: `format.rs`, `runtime.rs`, `cook/mod.rs`, `cook/model.rs`,
  `cook/stats.rs`, `tests/template.rs`.
- og-host-web `lib.rs`.
- packages/engine `src/web/worker.ts` and `tests/web/page.ts`.
- packages/knowledge `skills/og-game/SKILL.md`.
- docs/design.md.

`git merge-tree` against origin/main at `e568f91f` (after render-light):
the code merges cleanly, and only the generated wiki pages conflict
(`api.ogt` line numbers). Regenerate them with `og api gen --wiki` after
the merge.

## Metal references the coordinator must re-record

- fx-showcase geometry station frames: the new path tube, and the points
  tubes respaced by arc length.
- og-tests `geometry-procedural`: its tube is respaced.
- templates/game intro frames. The new music track is sound only, so the
  frames are likely unmoved; confirm.
- The video fixture's `references/metal/video-90.png`: likely unmoved.
  Item 5 changes which texture the sprite samples, not its content.
- No Metal reference exists for the fixture's `pair` and `surfaces` frames.
  Their tests compare against the source video through ffmpeg; a Metal
  reference is optional.

## Left out

- The browser page test of web frame capture (`tests/web/page.ts`) was
  written but not run, because there is no wasm build in this round. The
  wave-end run must exercise it.
- There is no size threshold for cooking a BVH: every model cooks one
  unless it says `raycast false`. Whether the default should stay on at
  +64 % for a large model like the dropship is the coordinator's call.
- The mesh path of a video is about 4 % darker than the UI path (tone
  mapping of an unlit material). This is expected engine behaviour and
  was not changed.
