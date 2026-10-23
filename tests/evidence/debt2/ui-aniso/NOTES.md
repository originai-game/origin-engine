# debt2/ui-aniso: anisotropic sprite sampling under og-ui's nine-slice, and the last "skin" lines (2026-10-23)

Branch `debt2/ui-aniso` from main ad903b18. I worked in an orb (Linux,
lavapipe), so I could not measure any Metal frame and every Metal
reference it moves is owed. I did not run `og test`, `scripts/check.sh`,
web-reference, a wasm build or a release build. The loop was:

- `cargo nextest run -p og-ui` and `-p og-render`, with `cargo clippy` and
  `cargo fmt --check` on og-render, og-ui and og-runtime;
- `og api gen` and `og api check`;
- `bun test` in packages/knowledge, packages/create, and packages/engine's
  look.test.ts (dev addon and dev `og`);
- Rumpus Isle's frames script through `og run --headless`, with dev addons
  built from the old and the new og-render.

## What landed

| Commit | What |
|---|---|
| a5208045 | og-render: the sprite sampler takes `anisotropy_clamp` 16 where the device has `ANISOTROPIC_FILTERING` (`RenderInfo::sprite_anisotropy`, `SPRITE_ANISOTROPY`). `RenderCapabilities.spriteAnisotropy` is new in api.ogt, its outputs are regenerated, and it carries the regression test. Font atlases, the decal atlas and the 2D lightmap move to a separate trilinear sampler. |
| 356d32dd | The eight "skin" lines: the tower-defense archetype (5) and packages/engine's look.test.ts (2) with its fixture base.ogt (1). create's test now refuses a UI "skin" in what a scaffold ships. |
| 44de12c5 | og-render: no sprite anisotropy on a software rasterizer (a fix to a5208045). The test now also checks that a 1:1 draw is exact. |
| this one | These notes. |

## Item 1: the root cause, as measured

The test (crates/og-ui/tests/gfx.rs,
`a_middle_squeezed_six_times_keeps_its_outline_as_dark_as_the_art`) uses
a mipmapped 64 × 64 sRGB art. The art has a 4-texel outline (40) on a
cream fill (240) in red. The test draws a 32 × 48-texel cut of the art
twice: once 1:1, and once with its 48 rows squeezed into 8 px (6×). It
then reads the darkest red across the outline in each.

| Sampler (lavapipe) | 1:1 outline | 6× squeezed outline | fill |
|---|---|---|---|
| Old: Linear mips, anisotropy 1 | 40 | **160** | 240 |
| Anisotropy 16 | 40 | **40** | 240 |

So the band below ui-prose's 2× bound is the mip level the trilinear
sampler picks from the larger of the two axes. With anisotropic
filtering, the squeezed piece reads the art's level-0 value.

**How I saw the test fail.** I forced the test's `aniso` to 16 and kept
ad903b18's og-render. The test failed with "the outline through a 6×
squeezed middle reads 160, the art's level 0 40 (fill 240)". With the new
sampler it passes with 40.

## What the first version broke, and the fix (44de12c5)

Two things went wrong with a5208045 as first written.

**Text.** With MSDF text drawn through the anisotropic sampler, og-ui's
suite failed four tests:

- bold ink spilled off its glyphs;
- screen-layers had RMSE 0.0112;
- text-kerning had RMSE 0.0162;
- scifi's pressed Primary label read 4.20:1, under the 4.5:1 bar.

Averaging a distance field along the long axis spreads the edge. Font
atlases therefore stay on a trilinear sampler. I moved the decal atlas
and the 2D lightmap to the same sampler, so that they keep their old
filter.

**lavapipe softens 1:1 draws.** Rumpus Isle's frames then moved
10,505 to 55,300 px (the largest change was 82 to 100 grey levels). The
whole difference was in the UI's outlines and icons; the 3D scene did not
move. A scratch test of the same art drawn 1:1 gave these rows:

- trilinear: `240 240 40 40 40 40 240`;
- anisotropic: `216 124 40 40 124 216`.

lavapipe's anisotropic filter (an elliptical weighted average) blurs even
an isotropic footprint. On that rasterizer, anisotropy would soften every
sprite, not only a squeezed middle.

So `sprite_anisotropy` is 1 on a software adapter (`is_software_adapter`:
lavapipe, llvmpipe, SwiftShader), and `spriteAnisotropy` reports it.
The test now always checks that the 1:1 piece reads the art exactly
(cream, then 4 × 40, then cream). With the gate removed, it fails on
lavapipe: "vulkan-cpu (anisotropy 16): the 1:1 piece reads [216, 124, 40,
40, 124, 216] across its outline". The squeeze half runs where
`sprite_anisotropy` > 1. On lavapipe it prints that it is skipped and why.

**Tiers.**

- WebGPU, Metal and Vulkan on a real GPU have `ANISOTROPIC_FILTERING`
  (wgpu reports it for WebGPU by default, for Metal always, and for
  Vulkan with `samplerAnisotropy`).
- WebGL2 has it through `EXT_texture_filter_anisotropic`, when the
  extension's maximum is at least 16 (wgpu-hal's gles adapter).
- Anywhere else, the 2× bound in og-ui's `nine()` stays. It is unchanged
  and its comment now says why.

## Frames read (lavapipe)

- **og-ui's 11 frames** (target/tmp/og-ui, the suite's own captures)
  against ad903b18. At 44de12c5 all 11 are pixel-identical (AE 0):
  glass, materials, sample-screen, screen-layers, text-bidi, text-edit,
  text-fields-2, text-glow, text-kerning, text-mixed-runs and widget-set.
  og-ui: 115 tests run, 115 passed, 2 skipped. og-render: 92 passed.
- **Rumpus Isle**: the whole `tests/frames.og.ts` script, 1280 × 720.
  After a5208045 (anisotropy on lavapipe) every frame moved as described
  above. After 44de12c5, all 11 frames (title, lobby, card, bump, splash, scores, berry, crumble, pause, settings-fr, podium) are pixel-identical to the ad903b18 addon's (AE 0).
- **The ui-kit style sheets were not run.** The sprite sampler on
  lavapipe is now the old descriptor, and the frames above confirm that.

## Metal references owed (the coordinator's)

On Metal the sprite sampler becomes anisotropic, so any frame with a UI
image or sprite minified unevenly can move. A nine-slice middle squeezed
in one axis gets a crisper outline: this is the ui-prose softness, gone.
The likely moves are:

- `examples/rumpus-isle/references/metal/`: lobby, card, bump, splash,
  berry, crumble, pause, settings-fr and podium (the small controls).
  Title and scores may move too.
- `crates/og-ui/references/metal/kit-style-{cartoon,fantasy,paper,pixel,scifi}.png`
  and their `-phone` sheets, plus `widget-set`, `sample-screen`,
  `materials`, `screen-layers` and the `text-*` frames that draw atlas
  frames. `kit-desktop-*`, `kit-phone-*` and `kit-locale-*` have no atlas
  and should not move.
- World sprites, sprite meshes and sprite arrays seen at an angle, or
  scaled unevenly, in 2D games and og-tests' sprites2d. Text, decals and
  the 2D lightmap keep their old filter and should not move.

None of these was measured. The coordinator should check the og-ui and
Rumpus frames on Metal first.

## Item 2: the skin lines

- tower-defense: in main.ts, `const skin` is now `const style`, passed as
  `{ style, … }`. ui.ts line 1 and line 354, and components.ts'
  `leftTitle` doc, now say "style".
- packages/engine: look.test.ts lines 7 and 89 and
  fixtures/look/assets/looks/base.ogt line 4 now say "style".
  look.test.ts passes (4 tests, dev addon).
- The test: packages/create's `what a scaffold ships never calls a UI
  style a skin`. It reads templates/game, game-2d and archetypes. It
  flags a sentence that says skin beside the UI's words (og.ui, ui,
  style, entrance, theme) when it names none of the word's other
  meanings. The list of other meanings is knowledge's `SKIN_ELSEWHERE`,
  now exported. On the old files it names the five tower-defense
  sentences. Character skin colours (runner, topdown) and "the orb's
  skin" (idle) pass. create: 15 tests pass; knowledge: 43 pass.

## Shared files touched

- `crates/og-render/src/lib.rs`: `RenderInfo::sprite_anisotropy`,
  `SPRITE_ANISOTROPY`, the sprite sampler's clamp, and the new
  `trilinear_sampler` for fonts, decals and the 2D lightmap.
  `crates/og-render/src/views.rs`: the view's fx group uses it.
- `crates/og-api/api.ogt` (`RenderCapabilities.spriteAnisotropy`) and its
  outputs: packages/engine/schema/api.schema.json, types/index.d.ts and
  the wiki. Every wiki page's `api.ogt line` stamps shifted by one.
- `crates/og-runtime/src/api.rs`: the capability (0 without a renderer).
- `crates/og-ui/src/ui.rs`: the `nine()` comment only.
  `crates/og-ui/tests/gfx.rs`: the test.
- `packages/knowledge/scripts/validate.ts` (`SKIN_ELSEWHERE` is exported)
  and `packages/create/tests/create.test.ts`.
- `docs/design.md` (UI) and the og-game skill (§8, sprites).

## Left out, and why

- **The 2× bound stays.** It is still what holds on WebGL2 without the
  extension and on software rasterizers. Lifting it to 1× (or removing
  it) where anisotropy is on would be a separate decision.
- **Anisotropy for text.** It measurably harms MSDF text.
- **The vulkan-cpu tier never runs the squeeze half of the test.** No
  lane in the orb proves the anisotropic path. It runs on the
  coordinator's Metal lane.

## For the coordinator to decide

1. Re-record the Metal frames above, and confirm on Metal that the test's
   1:1 check holds. If Metal's anisotropic filter also softens a 1:1
   draw, the test will fail there, and a lower clamp or a
   per-backend choice is needed.
2. Whether world sprites should share the anisotropic sampler (they do
   now: it is one sampler) or keep trilinear.
3. Whether to lift ui-prose's 2× bound on tiers with anisotropy.
