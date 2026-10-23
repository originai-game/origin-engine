# Hollowbell's skin: licences

A copy of the engine's pixel style pack (`templates/styles/pixel`),
its palette, ramp and fallback changed in `style.ogt` for Hollowbell.

| File | Licence | Origin |
|---|---|---|
| `atlas.png` | CC0 1.0 | self-made by the Origin Engine authors: drawn by `templates/ui-kit/tools/make-styles.py` (deterministic) |
| `icons.png` | CC0 1.0 | self-made: the kit's 32 icons (`tools/make-assets.py`) restyled by the same script |
| `style.ogt`, `*.ogt` | CC0 1.0 | written by the same script |
| `DotGothic16-Regular.ttf` | SIL Open Font License 1.1 (`OFL.txt`) | DotGothic16 (github.com/fontworks-fonts/DotGothic16); subset to Latin-1 with fontTools. Copyright 2020 The DotGothic16 Project Authors (https://github.com/fontworks-fonts/DotGothic16/) |

Characters these fonts lack (Chinese) are drawn with the skin's `fallback`
font, the game's own `fonts/NotoSansSC-hollowbell.ttf` (OFL 1.1,
`assets/fonts/OFL.txt`; `tools/subset-font.py`).
