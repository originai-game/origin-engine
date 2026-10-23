# Depth S8 `render-targets`: evidence

Branch `depth/render-targets`, rebased on main c5aa7ff1. All frames were
drawn on lavapipe (tier `vulkan-cpu`, llvmpipe LLVM 15, 1280×720) and read
one by one. They are copied into this directory.

## What landed

- **Render targets** (og-render `targets.rs`): `og.render.renderTarget(world,
  { width, height, format: ldr|hdr, depth })` answers `{ target, texture,
  depthTexture, width, height }`. The colour is an ordinary Texture, so any
  material map, og.Sprite, `og.render.sprite` or og.ui image shows it.
  - Depth textures are drawn on the full tier only. WebGL2 reports this as
    `RenderCapabilities.renderTargetDepth = false`; the member is never
    missing.
  - Capabilities also gain `maxRenderTargetSize` and `maxCustomPosts`.
- **Views** (`views.rs`): `og.CameraView` has og.Camera's fields plus target,
  viewport rect, layers, order, clear and clearColor, every, and active.
  `og.Mirror` is a mirrored view using a Kooima off-axis frustum fitted to
  the quad.
  - Each view is a timed graph pass named `view-<i>`, drawn before the main
    pass. Viewport views resolve after the post chain, under lines, the
    overlay and the UI.
  - Snapshot and restore: `Renderer::restore_state` marks every view stale,
    so a target drawn with `every > 1` redraws on the next frame and never
    shows the image of a cut run.
  - Capture reads the main target, and views draw before it, so a capture
    shows this frame's targets.
- **Layers**: `layers u32` (default 1) on og.Mesh, og.Model, og.Sprite,
  og.Lines, og.Points and the og.Camera resource, plus on CameraView and
  Mirror.
  - The semantics are three.js's and match S1's raycast exactly: bit n is
    layer n, the default 1 is layer 0, and an object draws when
    `object & camera != 0`.
  - `og.render.raycast`'s `layers` now tests each og.Mesh's or og.Model's own
    mask with the same `&`. The old "layer 0 only" stub in og-runtime
    `geometry.rs` is gone.
  - Shadow maps take every caster whatever its layers.
  - The S5 id pass (`id_pass.rs`) skips instances off the main camera's mask,
    so it keeps describing the main view.
- **Full custom shaders** (`custom.rs`): `type shader` with `mode full lit|unlit`
  defines `vertex(v, model, p)` and `fragment(f, p)` entries.
  - The binding contract is `camera()`, the model matrix (skin folded in,
    morphs applied), `Params`, `tex0`..`tex7` (the material's eight slots),
    and the helpers `standard_vertex`, `standard_surface` and `noise3`.
  - The prepass, every shadow pass, the water reflection and every view
    draw through the same vertex entry and cut by alpha.
  - og-cook validates the lit and shadow modules for WGSL, SPIR-V, MSL and
    GLSL ES with naga.
- **Custom post passes** (`post_custom.rs`): the `type post` asset (an
  og.PostPass blob, naga-validated per tier) loads with `og.load.post`.
  - `PostEffects.custom1..4` is the ordered list of Post handles. The
    passes run as `custom-post.<i>` after motion blur and before exposure,
    bloom and tonemap.
  - Parameters are set with `og.render.setPostParam`.
  - A pass the device refuses to compile is skipped and reported by
    `og.render.postErrors`.
- **IDL, both hosts and the TS package**: everything above is in `api.ogt`,
  in the node and web host traits, and in `packages/engine/src/targets.ts`.
  The column factories RenderTargetFormat, RenderTarget and Post are added.
  `og api check`, `cargo test -p og-idl` and `bun test
  tests/api-surface.test.ts` pass.
- **fx-showcase**:
  - New `views` station (src/views.ts, x 3400) with playtest `tests/views.og.ts`.
  - The `shaders` station now has a full-shader dissolve panel and a CRT
    `type post` pass over the whole station
    (`assets/shaders/dissolve-full.*`, `assets/post/crt.*`). Its playtest
    `tests/shaders.og.ts` is extended.
- **Docs**: design.md §2.6 gains targets, views, layers, custom shaders and
  post passes. The og-game skill gains §22; "Where to look things up" is now
  §23. The knowledge tests pass.

## Frames read

| frame | what it shows |
|---|---|
| `views-monitor.png` (gfx) | The room with a monitor on the back wall. The monitor shows the room from a corner camera, including the magenta cube on layer 1, which the main view does not draw anywhere. |
| `views-mirror.png` (gfx) | The left-wall mirror (hdr target as emissiveMap) reflects the green pillar, red box and blue sphere at the correct mirrored positions. |
| `views-split.png` (gfx) | The front view fills the left half and the top-down view fills the right half, with a clean seam. |
| `shader-full-dissolve.png` (gfx) | Noise holes with glowing edges through the sphere and the thin panel. The panel's shadow on the floor has matching holes. |
| `post-crt.png` (gfx) | Barrel-bent image with black corners, visible scanlines, an RGB column mask and a vignette. |
| `station-views.png` (`og run --headless`, station views) | Mirror on the left wall reflecting the walking walker, the pillar and the ball. The monitor on the back wall shows the corner camera's view with the magenta layer-1 crate, which is absent from the main view. A split view from above sits bottom right. |
| `station-shaders.png` (`og run --headless`, station shaders) | The whole frame is bent by the CRT with black corners and scanlines. The hooked dissolve sphere and swaying grass are unchanged. The full-shader panel is cut by noise, and its shadow on the ground carries the same holes. |

Station numbers, measured on these frames with the playtests' thresholds:
- 48 magenta pixels on the monitor, against 4 elsewhere (anti-aliased edge
  noise).
- Mirror mean RGB 138/153/148.
- 547 blue pixels of the ball inside the split rect.
- Frame-to-frame motion: sphere 15.6, grass 8.0, full-shader panel 4.3.
- All four CRT corners read 0.

Gfx tests in `crates/og-tests/tests/targets.rs` and their reference names:
`views-monitor`, `views-mirror`, `views-split`, `shader-full-dissolve` and
`post-crt`. The frame-free checks pass on lavapipe:
- `a_layer_the_main_camera_lacks_shows_only_on_the_monitor`;
- `a_full_shaders_shadow_follows_its_cut`: 14365 shadowed pixels whole
  against 9518 dissolved;
- `a_failing_post_pass_is_reported_and_skipped`.

The five reference comparisons fail only because no reference exists yet.

## Metal references owed (coordinator)

- gfx: `views-monitor`, `views-mirror`, `views-split`, `shader-full-dissolve`,
  `post-crt`.
- fx-showcase: `views` (new) and `shaders` (changed: the CRT pass and the
  full-shader panel move every pixel).

## Shared files touched

- `crates/og-api/api.ogt`: `layers` fields on existing records,
  `PostEffects.custom1..4`, the RenderCapabilities members and the raycast
  doc. Everything else is one appended block at the end of `module render`,
  plus `load.post`.
- og-render:
  - Frame graph: `lib.rs` (FramePass::View / CustomPost, view planning,
    pipelines for view-only skinned batches, `restore_state`, `RenderInfo.max_texture_size`)
    and `post.rs` (custom-post transients and passes).
  - `scene.rs`: the layer, view and post fields.
  - Also `instances.rs`, `hooks.rs` (ShaderMode), `shaders.rs`,
    `texture_array.rs`, `video.rs`, `id_pass.rs` and `tests/webgl2.rs`.
- og-cook: `shader.rs`, `cook/mod.rs`, `format.rs`, `runtime.rs`.
- og-runtime: `extract.rs` (MeshRow.layers, camera layers, the
  `views::extract` call), `engine.rs`, `api.rs` (capabilities), `codec.rs`,
  `feel.rs` (sprite layers), `geometry.rs` (raycast and line/point layers),
  `stream.rs`, and the new `views.rs`.
- Hosts: og-host-node and og-host-web `host.rs` and `lib.rs`
  (`RenderBackend::post_errors`).
- og-tests: `scenes.rs`, `atmo_scenes.rs`, `p2_scenes.rs` (`mode` on
  literals), and the new `target_scenes.rs`.
- Showcase: `src/main.ts`, `src/e5.ts`, `tests/shaders.og.ts`, and the new
  views files.

## Left out and limits

- A view draws only opaque meshes and models (skinned included), terrain
  and sky. It draws no transparents, sprites, particles, text, lines or post
  effects; those belong to the main camera. Screen sprites draw on the
  main frame only.
- A frame with views culls on the CPU, because GPU culling builds one
  camera's list.
- Target depth is full tier only (WebGL2 reports it in capabilities).
- `PostEffects.custom` is four fixed slots (`custom1..4`), not a list
  column: the IDL has no list fields in records. og-render accepts up to
  eight.
- Not run, per the rules: the gate, `og test`, web-reference and the
  wasm/release builds. The playtests were checked against the og-run frames
  with a scratch script, not through `og test`.
