# r4/render-billboards

Branch `r4/render-billboards` from `15ca814c` (`origin/r4/candidate`).
The og-render README's owed item "2D: billboards (camera-facing world
sprites; particles have them)" is delivered.

## What landed

- **A mode of og.Sprite, not a component.** IDL enum `SpriteFacing`
  (`none | camera | cameraY`) and its column; fields `og.Sprite.facing` and
  `SpriteDesc.facing`; the `SpriteFacing` factory exported by hand from
  `packages/engine/src/index.ts`. The decision and its consequences are in
  the enum's doc in `api.ogt`: a billboard is the same quad, texture or
  array layer, flash, glow, blend, 2D lighting, drop shadow and runs as any
  sprite, so there is no second draw path.
- **og-render.** `RenderScene::sprite_facing` (by sprite index, like
  `sprite_layers`). The sprite instance carries the mode (location 8, 100
  → 104 bytes); `sprite.wgsl` and `sprite_array.wgsl` stand each corner
  with `stand`: 0 the old xy expression, 1 the camera's right and up, 2
  its right laid level and +y, 3 (internal) the sun-facing card for a
  faced sprite's drop shadow. The camera's axes are two vec4s at the end
  of the frame uniform (`view_right`, `view_up`), written for the main
  camera, the planar reflection, every `View` and every reflection probe
  face, so every view that draws sprites turns them to its own eye, on
  both tiers (render passes only; every module validates for MSL, SPIR-V
  and GLSL ES 3.0 in `tests/shaders.rs`).
- **Sorting**: a faced sprite without a `sprite_order` key is keyed by its
  negated view depth within its layer (back to front, after the layer's
  drop shadows, before its key-0 flat sprites). Each due probe now builds
  its own sprite list sorted by distance from the probe.
- **Culling**: a sphere round the anchor through the farthest corner,
  against the main camera's and each view's frustum
  (`FrameStats::sprites_culled`, the main camera's). Flat sprites are not
  culled, as before.
- **Drop shadows** of faced sprites are cast by the quad stood upright
  facing the sun (`sun_card_right`), the same from every camera;
  `ShadowGround::fit_facing` bends on that card and og-runtime's terrain
  shadows use it.
- **Lit and unlit**: 2D lighting, fog, glow and the untonemapped path are
  the sprite shader's and apply unchanged.
- **Deterministic**: the turn is computed on the GPU per camera; the CPU
  id pass (`id_pass.rs`) mirrors it for picking. Nothing the simulation
  reads changes.
- og-runtime: rows and `og.render.sprite` hand the facing on; screen
  sprites ignore it; a faced nine-slice's pieces are re-anchored at the
  panel's anchor (`Sprite::anchored_at`) so it turns as one panel; an
  og.Sprite's afterimages face with it, its blob shadow stays flat.

## Frames recorded

`crates/og-tests/references/vulkan-cpu/sprites-billboard.png` (sha256
`54bfe2ba9ed3d0d1…`), lavapipe (llvmpipe, LLVM 15.0.6), 1280×720, aa off.
What I read in it: a checker ground seen from above and to the right,
a grey crate; a flat red figure (`none`) sheared by the view angle; two
pixel trees and a blue figure standing upright (`cameraY`) facing the
camera, each with a dark drop shadow cast up-right along the sun showing
the whole tree or figure; the red-ringed sign with its bar turned 0.35 rad
floating in the view plane (`camera`); a green figure (layer 1, 2D-lit)
warmed by an orange light at its side; two translucent puffs, the red one
farther and the blue one nearer drawn over it although submitted in the
other order; a picture-in-picture view from the side (top right) where
the trees, the green figure and the sign face that camera, and the flat
red figure is seen edge on. The ninth sprite, a tree behind the camera,
is culled.

**Metal reference is owed** (the coordinator's Mac lane): record
`OG_GFX_CAPTURE=sprites-billboard` there. Until then the other tiers
compare against nothing for this scene except vulkan-cpu.

## Tests that fail on the old code

- og-tests `billboards_face_each_camera_sort_back_to_front_and_cull_by_bounds`:
  with every facing forced to `None` (the old renderer's behaviour) the
  frame misses the reference (RMSE 0.053, 2.05 % of pixels past 32),
  nothing is culled (9 sprites drawn, 0 culled), and swapping the two puffs
  moves 4,661 pixels.
- og-render `faced_sprites_sort_back_to_front_cull_to_the_frustum_and_cast_sun_facing_shadows`,
  `a_sprite_anchored_elsewhere_keeps_its_corners`; og-runtime
  `tests/billboards.rs` (new fields; would not compile before).

## What moves

- Native frames of the existing sprite scenes, old vs new on lavapipe
  (`og-gfx <scene> --out`): `sprites-shadow`, `sprites-array-fx`,
  `text-msdf` byte-identical; `sprites-20k` moves 444 pixels by 1/255 (RMSE
  4e-5; the old code reproduces itself byte for byte, so this is the
  compiler's code for the changed vertex shader, as the slope path once
  moved it on Metal). Within every tolerance; the Metal lane may see the
  same kind of move.
- The sprite instance stride (100 → 104) and the frame uniform (+32
  bytes).
- og.Sprite gains a field, so the registry and a world's `world_hash`
  over records change (nothing in the repository pins one); generated
  `.d.ts`, schema, bindings, `host.rs` and wiki (line numbers mostly).

## Decisions for the coordinator

1. Mode on og.Sprite over a separate component (above). World text does
   not face yet.
2. Billboards sort among themselves back to front but, like every sprite
   and particle, draw after the transparent meshes: a billboard behind a
   glass pane still draws over it. Interleaving sprites into the
   transparent mesh sort would be a larger change to the main pass.
3. A faced sprite's drop shadow uses a sun-facing card rather than the
   camera-facing quad, so shadows don't swing with the camera and agree
   across views.
4. Culling is only for faced sprites (their bounds are camera-dependent
   anyway); flat sprites stay unculled as before.

## Not run

`scripts/check.sh` (any lane), `og test`, web-reference, any wasm or
release build, any browser tier, the Metal capture, og-tests suites other
than `gfx billboards` and `sprites2d` (notably `targets.rs` and
`probes.rs`, whose views and probe faces now carry the new uniform
members), and other templates' frames. Run: `cargo fmt`, `cargo clippy
--all-targets -D warnings`, `cargo test -p og-render` (all), `-p og-idl`,
og-runtime `--test billboards`, og-tests `--test gfx billboards` and
`--test sprites2d`, `og api gen` / `api check`, `bun test
tests/api-surface.test.ts` (dev addon and og copied into
`packages/engine/native` by hand), `bun run test` in packages/knowledge.
