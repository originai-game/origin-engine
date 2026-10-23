# Spark Orb's skin (the engine's neon style pack, recoloured): licences

The palette, the body size and the fallback font in `style.ogt` are this
game's edits by hand; `atlas.png` is the pack's atlas with its magenta
frames turned violet by `tools/make-assets.py`; the rest is the pack as
`templates/styles/neon` ships it.

| File | Licence | Origin |
|---|---|---|
| `atlas.png` | CC0 1.0 | self-made by the Origin Engine authors: drawn by `templates/ui-kit/tools/make-styles.py` (deterministic) |
| `icons.png` | CC0 1.0 | self-made: the kit's 32 icons (`tools/make-assets.py`) restyled by the same script |
| `style.ogt`, `*.ogt` | CC0 1.0 | written by the same script |
| `ChakraPetch-BoldItalic.ttf` | SIL Open Font License 1.1 (`OFL.txt`) | Chakra Petch Bold Italic (github.com/m4rc1e/Chakra-Petch); unmodified. Copyright 2018 The Chakra Petch Project Authors (https://github.com/m4rc1e/Chakra-Petch.git) |
| `Orbitron-Medium.ttf` | SIL Open Font License 1.1 (`OFL.txt`) | Orbitron Medium (github.com/theleagueof/orbitron); unmodified. Copyright 2018 The Orbitron Project Authors (https://github.com/theleagueof/orbitron), with Reserved Font Name: "Orbitron". |
| `Rajdhani-Bold.ttf` | SIL Open Font License 1.1 (`OFL.txt`) | Rajdhani Bold (Indian Type Foundry, Google Fonts); subset to Latin-1 with fontTools. Copyright (c) 2014 Indian Type Foundry (info@indiantypefoundry.com) |

Characters these fonts lack (Chinese, Japanese, Korean) are drawn with the
skin's `fallback` font, `fonts/NotoSansSC-subset.ttf` (Noto Sans SC subset to
this game's Chinese strings, OFL 1.1, `assets/fonts/OFL.txt`).
