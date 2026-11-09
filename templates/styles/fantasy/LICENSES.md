# fantasy: licences and provenance

The style (`style.ogt`, `choices.json`, `art-requests.json`, the `sfx-*.ogt`
presets the cook synthesises) is the engine's own work, under the engine's
licence. It was started by `og style new` and authored from there.

| File | Licence | Origin |
|---|---|---|
| `art/*.png` (frames, buttons, ornaments, knobs) | CC0 1.0 | Painted for this pack by the Origin generate service (`gpt-image-2.6-sunburst`), one original image per slot from the prompt in `art/provenance.json`, with no reference image; trimmed to its alpha and downscaled to 3.5 texels per CSS px of its cell. No commercial game's art, brand or layout was used. |
| `atlas.png`, `atlas@2x.png`, `atlas@3x.png` | CC0 1.0 | Packed from `art/` by `og style art` (deterministic). |
| `icons.png` | CC0 1.0 | The standard 32 icon names, painted for this pack by the same service in four sheets of eight (prompts below), cut into the 8×4 grid and turned into a white mask that keeps the painted shading, so the UI tints it with its ink. |
| `*.png.ogt` | CC0 1.0 | Import settings. |

It draws with the shared fonts, which stay in `templates/styles/shared` and
`templates/styles/cjk` with their licences: `Newsreader-Medium.ttf` (body),
`Cinzel-Bold.ttf` (display), `Spectral-SemiBold.ttf` (numeric) and the
Noto Sans SC faces.

The icon sheets' prompt, each with its eight names in reading order:
"Eight separate UI icons on a transparent background, arranged in a grid of
4 columns and 2 rows with generous equal spacing, in reading order left to
right then top to bottom. One consistent original icon set for a dark
fantasy RPG interface: each icon a bold, simple silhouette carved in warm
parchment-ivory with a thin dark brown outline and subtle hand-painted
shading, front view, readable at 24 pixels. Transparent background. Exactly
eight icons, no extra objects. No text, letters, numbers, logos or
watermark."
