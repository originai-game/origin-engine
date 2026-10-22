# Lamp Dive's skin (the engine's pixel style pack, recoloured): licences

The palette, the body text's 1 px dark shadow and the fallback font in
`skin.ogt` are this game's edits by hand; the rest is the pack as
`templates/ui-kit/assets/skins/pixel` ships it.

| File | Licence | Origin |
|---|---|---|
| `atlas.png` | CC0 1.0 | self-made by the Origin Engine authors: drawn by `templates/ui-kit/tools/make-skins.py` (deterministic) |
| `icons.png` | CC0 1.0 | self-made: the kit's 32 icons (`tools/make-assets.py`) restyled by the same script |
| `skin.ogt`, `*.ogt` | CC0 1.0 | written by the same script |
| `DotGothic16-Regular.ttf` | SIL Open Font License 1.1 (`OFL.txt`) | DotGothic16 (github.com/fontworks-fonts/DotGothic16); subset to Latin-1 with fontTools. Copyright 2020 The DotGothic16 Project Authors (https://github.com/fontworks-fonts/DotGothic16/) |

Characters these fonts lack (Chinese, Japanese, Korean) are drawn with the
skin's `fallback` font, `fonts/NotoSansSC-subset.ttf` (Noto Sans SC subset to
this game's Chinese strings, OFL 1.1, `assets/fonts/OFL.txt`).
