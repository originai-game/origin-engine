# scenario-pixel-vfx

Scenario's Pixel VFX Forge (https://www.scenario.com/explorations/pixel-vfx-forge/),
its renderer as shipped in the `scenario-pixel-vfx` agent skill and on the
exploration's page (the same `engine.js`, `color.js` and `forms.js` in both):

- the skill ZIP:
  https://iieqkinwjorxrzjp.public.blob.vercel-storage.com/explorations/f5284ec0200005bb234c2b9ddafa82cd3ab54b3321dda785ce8cccd17d23cd2c/scenario-pixel-vfx-skill.zip
  (SHA-256 `f5284ec0200005bb234c2b9ddafa82cd3ab54b3321dda785ce8cccd17d23cd2c`);
- the making-of PDF:
  https://iieqkinwjorxrzjp.public.blob.vercel-storage.com/explorations/4d09a2a612b5e2bc12c30a62d72334f11232891b0cbc9d012c16c7a34fbbb4e8/pixel-vfx-forge-making-of.pdf
  (SHA-256 `4d09a2a612b5e2bc12c30a62d72334f11232891b0cbc9d012c16c7a34fbbb4e8`);
- Scenario's skills repository, which the page links: https://github.com/scenario-labs/skills.

Permission: none of the sources states a licence. The engine's owner
confirmed with Scenario that the code may be used directly; this file is
the record of that permission.

Ported or taken:

| Source (`scenario-pixel-vfx/scripts/lib/`) | SHA-256 | Here |
|---|---|---|
| `engine.js` (ENGINE_VERSION 1): hashing, mulberry32, gradient noise and fbm, the raster helpers, the 3x5 font, the 22 layer types, `normalize`, `renderFrame`, `paint`, `post` (clean-up, outlines, palette lock) and `render` (the impact frame) | `c57bb2079793aa22ef6578f7a6d6d55854903bf4c846b3beeb6ced67ede7d9e0` | ported to Rust: `crates/og-cook/src/cook/vfx2d/engine.rs` (the recipe, canvas, noise and passes) and `vfx2d/layers.rs` (the layer types, shapes and font) |
| `color.js`: hex parsing, OKLab and OKLCH, `rampFromHue`, `shiftRamp`, the named ramps and palettes, `resolveRamp`, `nearestInPalette`, `darken` | `ec031a53ef650c0f99c80db3bbe8ee1b426ca883de1877ff9f6fe0b606ba7512` | ported to Rust: `crates/og-cook/src/cook/vfx2d/color.rs` |
| `forms.js`: `LIBRARY` (42 prompts and seeds) and `interpret` | `92c27a7deaf85a4517f7fcfdf4a35a631e64899dc5b69cdbe303f732b1853762` | not ported; run once to write the library's recipes as `interpret(prompt, seed).recipe` gives them (`JSON.stringify(recipe, null, 2)`), two of which are test data: `crates/og-cook/tests/fixtures/vfx2d/` (`meteor-strike`, `level-up`) |
| `references/recipe.md` | | the recipe and layer reference behind `crates/og-api/guides/pixel-vfx.md` (rewritten) |

JavaScript's `Math` is matched by `crates/og-cook/src/cook/vfx2d/jsmath.rs`
(V8's `Math.hypot`; the `libm` crate for the transcendental functions) and
the Forge's `Float32Array` heat by f32 storage. Checked against Node 26.3
(V8 14.6) running `engine.js`: the 42 library recipes and 126 variants with
palettes, outlines, dither, hue and saturation, glow, gamma and `blend
under` render byte-identical.
