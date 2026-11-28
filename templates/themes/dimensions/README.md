# Theme dimensions

A theme recipe picks one value per dimension; each value is one JSON file
here, `<dimension>/<value>.json`, and the recipe compiler turns the picks
into a pack layer. The taste lives in these files, not in engine code.
Every file has an `about` line. og-cook's
`the_theme_dimensions_name_real_things_and_draw` test checks every value:
the files it names exist and are pinned, fonts carry the family names the
files give, keys are ones the default pack declares, and the art draws.

| Dimension | Keys | Compiles into |
|---|---|---|
| `material` (glass, metal, paper, plastic, flat, matte, wood, cloth, stone, pixel, ink) | `panel` (optional: a header band, `{"header": {height, color, rule, lift}}`), `art` (the generator's material layers, `templates/tools/theme-art.ts`), `scheme` (the scheme it is drawn for), `faces` (per surface: OKLCH lightness `[top, bottom]`, chroma, and hue `n` neutral or `a` accent), `primary` (the primary button's lightness band), `text` (the lightness band a face under text keeps) | the art spec's `material`; the faces' palette |
| `shape` | `art` (the generator's shape: `family`, `radius`, `cut`, `step`, `wobble`), `radius` (the eight radius tokens) | the art spec's `shape`; `theme.radius` |
| `type` | `fonts` (files), `families` (each role's family, as the file names it), `cjk` (the CJK fallback family), `fontFamily` (the pack's four lists), `tracking` (fixes per step, merged over the default's) | `fonts`, `theme.fontFamily`, `theme.tracking` |
| `motion` | `variants` (every motion slot's classes) | `variants` |
| `ornament` | `art` (one ornament: `kind`, `size`, `alpha`, `inset`); a recipe names one per slot | the art spec's `ornament` |
| `effects` | `textures` (particle textures), `textGlow`, `shadow` (`glow`, `drop`, `soft`, `none`), optional `ink` and `pixel` | the effect presets' textures, glow and shadow tokens |
| `icons` | `set` (the icon set), `colors` (the `icon-*` tokens; a bare name is a colour role) | `iconSet`, `theme.colors` |
| `sound` | `sounds` (per event: `clips`, `volume`, `pitch`), in pack format | `sounds` |
| `density` | `density` (`compact`, `comfortable` or `touch`) | the root's `data-density` |

Colour refs inside `art` are palette roles, `top` and `bottom` (the piece's
face), hex, or `{"mix": [a, b, t]}`; a material's grain and rim tones mix
from its own face and the outline, so a material needs no extra roles.

The sound sets' clips are in `templates/styles/shared/sfx/<set>/`, mastered
by `templates/tools/master-theme-sounds.ts` (prompts, sources and levels).
