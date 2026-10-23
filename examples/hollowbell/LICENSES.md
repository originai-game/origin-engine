# Asset licences

Nothing in this project is downloaded: every asset is drawn or synthesised
by `tools/make-assets.ts` (deterministic, a seeded generator), written by
hand, synthesised by `og cook` from `og sfx` presets, or one of the
engine's own CC0 / OFL assets.

| File | Licence | Origin |
|---|---|---|
| `assets/palettes/hollowbell.hex` | CC0 1.0 | self-made: the game's 32 colours, `tools/make-assets.ts` |
| `assets/tiles/hollow.png`, `hollow.ogt` | CC0 1.0 | self-made: the tileset (the rock's 47 blob shapes, the back wall, planks, a torch, crystals, grass, hills, a fence, thorns) drawn pixel by pixel, `tools/make-assets.ts` |
| `assets/maps/hollow.tmj` | CC0 1.0 | self-made: the Tiled map and its objects, `tools/make-assets.ts` |
| `assets/sprites/*.png`, `*.ogt`, `assets/sprites/items/*.png` | CC0 1.0 | self-made: Wren, the beetle, the moth, Old Burrow, the bell, the sky, the items' texture array, `tools/make-assets.ts` |
| `assets/fx/**` | CC0 1.0 | self-made: sparks, dust, the motes' texture array and the particle emitters, `tools/make-assets.ts` |
| `assets/audio/*.ogt` | CC0 1.0 | `og sfx` presets (seed, pitch, length, volume chosen here) that `og cook` synthesises |
| `assets/audio/bell.wav` | CC0 1.0 | self-made: a synthesised bell toll, `tools/make-assets.ts` |
| `assets/music/hollow.wav` (not committed) | CC0 1.0 | self-made: "Under the Hill", synthesised by `tools/make-assets.ts` |
| `assets/looks/*.ogt`, `assets/strings/*.ogt`, every other `.ogt` | CC0 1.0 | self-made, written by hand |
| `assets/fonts/NotoSansSC-hollowbell.ttf` | SIL Open Font License 1.1 (`assets/fonts/OFL.txt`) | Noto Sans SC Regular, subset to the CJK characters of `assets/strings/*.ogt` by `tools/subset-font.py` from the engine's `templates/game/assets/fonts/NotoSansSC-Regular-subset.ttf` |
| `assets/styles/hollowbell/*` | CC0 1.0 art, SIL OFL 1.1 font (DotGothic16) | a copy of the engine's pixel style pack (`templates/styles/pixel`), recoloured; `assets/styles/hollowbell/LICENSES.md` |
| `references/*/*.png` | CC0 1.0 | frames of this game recorded by `tests/frames.og.ts` |
