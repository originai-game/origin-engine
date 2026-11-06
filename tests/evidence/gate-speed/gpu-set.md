# Which tests need the GPU (2026-11-06, macrunner, lane-22 at 1e399afc9)

Measured by running everything with `OG_RENDERER=none` and, for bun, also
with the Metal renderer, then comparing each test's result and its
`expect()` count (bun's junit report): a test is in the GPU set when it
fails without a renderer or checks less without one.

## Rust: 546 tests

`OG_RENDERER=none cargo nextest run --workspace --no-fail-fast`: 2793 run,
2247 passed, 546 failed, every failure NoAdapter. og-tests 487 (tests 472,
render 15), og-tests-cooked 31, og-ui 14 (gfx 11, kit_gfx 2,
screen_layers 1), og-cook 7, og-render 5 (shared_frames 3, surface 2;
profile2d is feature-gated), og-motion::gpu 2. The last two pass silently
or on lavapipe before 97df133cb. A nextest filter for the set:

```
package(og-tests) | package(og-tests-cooked) | binary_id(og-motion::gpu)
| (package(og-render) & test(/^(shared_frames|surface|profile2d)::/))
| (package(og-ui) & test(/^(gfx|kit_gfx|screen_layers)::/))
| (package(og-cook) & (test(/^compressed::/) | test(/^template::cooked_models_(render|draw)_/)
  | test(=anim_compress::compressed_clips_stay_within_tolerance_and_shrink)
  | test(=textures_p4::a_texture_array_cooks_its_layers_and_uploads_them_as_one_array)))
```

## Bun (packages/engine): 36 tests

692 tests; without a renderer 86 s wall (4 workers, load ~10), with Metal
248 s. 26 fail without a renderer: render 9, feel 3, terrain 3,
polish 4, og-render 2, look 1, lanes 1, asset-stream 1 (three groups),
template 1 (8 checks, not 9), terrain-stream 1. Nine pass but check less
(an early return when `run.tier === "none"`): og-video 5 (of 5; also need
ffmpeg), describe-ui 1, trace 1, uncooked-release 1, polish's "og ui
check draws a phone cell at 2 device pixels" 1. And no-renderer's
"og test without a renderer passes…": its lit half unsets OG_RENDERER, so
on an orb it runs on lavapipe. knowledge, create and design have none.

Not GPU, open on main: ui-hud's unitFrame/partyFrames and speedometer
(texts come back as glyph-ID code points); `og build --publish ships each
group` at 120 s under load only.
