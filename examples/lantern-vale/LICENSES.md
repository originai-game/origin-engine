# Asset licences

Nothing in this project is downloaded: every asset is made by
`tools/make-assets.ts` (deterministic, a seeded generator), written by
hand, synthesised by `og cook` from `og sfx` presets, or one of the
engine's own CC0 / OFL assets.

| File | Licence | Origin |
|---|---|---|
| `assets/models/*.vox`, `*.vox.ogt` | CC0 1.0 | self-made: the hero, the lantern, a tree, a boulder, the far side's pine and arch, voxel by voxel in `tools/make-assets.ts` |
| `assets/models/slime.glb` | CC0 1.0 | self-made: the slime's voxels meshed into a skinned glTF with two clips by `tools/make-assets.ts` |
| `assets/textures/*.png` | CC0 1.0 | self-made: tileable grass, path, rock and moss, a spark and the far side's banner, `tools/make-assets.ts` |
| `assets/sprites/faces/*.png` | CC0 1.0 | self-made: the slimes' expression bubbles, `tools/make-assets.ts` |
| `assets/terrain/vale.png`, `vale-control.png` | CC0 1.0 | self-made: the valley's 16-bit height field and splat map, `tools/make-assets.ts` |
| `assets/env/dusk.hdr` | CC0 1.0 | self-made: a 64 x 32 synthetic dusk sky, `tools/make-assets.ts` |
| `assets/music/dusk.wav` (not committed) | CC0 1.0 | self-made: the 4-minute music, synthesised by `tools/make-assets.ts` |
| `assets/audio/*.ogt` | CC0 1.0 | `og sfx` presets that `og cook` synthesises |
| `assets/fx/burst.ogt`, `assets/scenes/main.ogt`, `assets/looks/*.ogt`, `assets/strings/*.ogt`, every other `.ogt` | CC0 1.0 | self-made, written by hand |
| `assets/fonts/NotoSansSC-subset.ttf` | SIL Open Font License 1.1 (`assets/fonts/OFL.txt`) | Noto Sans SC Regular, subset to the CJK characters of `assets/strings/*.ogt` by `tools/subset-font.py` from the engine's `templates/game/assets/fonts/NotoSansSC-Regular-subset.ttf` |
| `assets/styles/lantern/*` | CC0 1.0 art, SIL OFL 1.1 fonts | a copy of the engine's fantasy style pack (`templates/styles/fantasy`), `assets/styles/lantern/LICENSES.md`; its `fgSubtle` darkened and its fallback this game's subset |
| `references/*/*.png` | CC0 1.0 | frames of this game recorded by `tests/frames.og.ts` |
