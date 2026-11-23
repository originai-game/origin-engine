# The shared fonts, icons and sounds: licences

Fonts and icon sheets the generated seed styles draw with (`og style new`,
docs/ui.md §3.3). They came from the paper and neon packs, which were
folded into storybook and cyber (the owner, 2026-11-02); this directory is
not a style.

| File | Licence | Origin |
|---|---|---|
| `ink-icons.png` | CC0 1.0 | self-made by the Origin Engine authors: the kit's 32 icons (`templates/ui-kit/tools/make-assets.py`) restyled in ink by `templates/ui-kit/tools/make-styles.py` (deterministic) |
| `glow-icons.png` | CC0 1.0 | self-made: the same icons restyled as glowing tubes by the same script |
| `*.ogt` except `sfx-*.ogt` | CC0 1.0 | import settings written by the same script |
| `icons/game.ogt`, `icons/README.md`, `icons/game/*.svg` of the hero icons (`coin`, `gem`, `star`, `heart`, `shield`, `energy`, `ammo`, `lock`, `key`, `chest`, `gift`, `crown`, `potion`, `ticket`, `xp`, `alert`, `new`, `rarity`, `trophy`, `hourglass`) | CC0 1.0 | self-made by the Origin Engine authors: drawn by `templates/tools/make-game-icons.py` (deterministic), redrawn by hand from a concept sheet generated with origin's image tools; no generated raster is shipped |
| `icons/game/*.svg`, the other icons | CC BY 3.0 (`third_party/game-icons/LICENSE`) | game-icons.net (github.com/game-icons/icons), each icon's author in `third_party/game-icons/ICONS.md`; paths scaled to the 24-unit grid and the background square dropped by the same script. Credited by the engine's credits (`engineCredits`) |
| `sfx/default/*.ogg`, and each pack's `sfx/*.ogg` (`styles/scifi`, `styles/fantasy`, `styles/cartoon`) | Engine licence | Generated for the engine with origin's `generate_sfx` (ElevenLabs text-to-sound, model eleven_text_to_sound_v3) under the Origin platform's commercial account; the prompts and each original's media id are in `templates/tools/master-ui-sounds.py`, which trims, fades, levels and encodes them (deterministic). No third-party recording is included. |
| `sfx-*.ogt` | Engine licence | Original engine sound-synthesis presets, moved from the removed clean style; no recordings are bundled. |
| `InstrumentSans-Regular.ttf` | SIL Open Font License 1.1 (`OFL.txt`) | Instrument Sans (github.com/Instrument/instrument-sans); unmodified. Copyright 2022 The Instrument Sans Project Authors (https://github.com/Instrument/instrument-sans) |
| `InstrumentSans[wdth,wght].ttf` | SIL Open Font License 1.1 (`OFL.txt`) | Instrument Sans, the variable font (wdth 75–100, wght 400–700) from google/fonts `ofl/instrumentsans`; unmodified. The default UI face. Copyright 2022 The Instrument Sans Project Authors (https://github.com/Instrument/instrument-sans) |
| `Newsreader-SemiBold.ttf` | SIL Open Font License 1.1 (`OFL.txt`) | Newsreader (github.com/productiontype/Newsreader); a static instance (wght 600, opsz 24) of the variable font, subset to Latin-1 with fontTools. Copyright 2020 The Newsreader Project Authors (http://github.com/productiontype/Newsreader) |
| `PTMono-Regular.ttf` | SIL Open Font License 1.1 (`OFL.txt`) | PT Mono (ParaType); unmodified. Copyright © 2010 ParaType Inc., ParaType Ltd. All rights reserved. |
| `ChakraPetch-BoldItalic.ttf` | SIL Open Font License 1.1 (`OFL.txt`) | Chakra Petch Bold Italic (github.com/m4rc1e/Chakra-Petch); unmodified. Copyright 2018 The Chakra Petch Project Authors (https://github.com/m4rc1e/Chakra-Petch.git) |
| `ChakraPetch-SemiBold.ttf` | SIL Open Font License 1.1 (`OFL.txt`) | Chakra Petch SemiBold (github.com/m4rc1e/Chakra-Petch, Google Fonts); unmodified. Copyright 2018 The Chakra Petch Project Authors (https://github.com/m4rc1e/Chakra-Petch.git) |
| `Rajdhani-Bold.ttf` | SIL Open Font License 1.1 (`OFL.txt`) | Rajdhani Bold (Indian Type Foundry, Google Fonts); subset to Latin-1 with fontTools. Copyright (c) 2014 Indian Type Foundry (info@indiantypefoundry.com) |
| `stencil-icons.png` | CC0 1.0 | self-made: the kit's 32 icons cut as spray stencils (bridged, edges worn) by `templates/ui-kit/tools/make-styles.py`; the wasteland seed's |
| `plate-icons.png` | CC0 1.0 | self-made: the kit's 32 icons struck as raised plates (a rim around a sunk face) by the same script; the foundry seed's |
| `BigShouldersStencilDisplay-SemiBold.ttf` | SIL Open Font License 1.1 (`OFL.txt`) | Big Shoulders Stencil Display (github.com/xotypeco/big_shoulders, Google Fonts); a static instance (wght 600) of the variable font, subset to Latin-1 with fontTools. Copyright 2019 The Big Shoulders Project Authors (https://github.com/xotypeco/big_shoulders) |
| `BarlowSemiCondensed-SemiBold.ttf` | SIL Open Font License 1.1 (`OFL.txt`) | Barlow Semi Condensed SemiBold (github.com/jpt/barlow, Google Fonts); subset to Latin-1 with fontTools. Copyright 2017 The Barlow Project Authors (https://github.com/jpt/barlow) |
| `BarlowCondensed-SemiBold.ttf` | SIL Open Font License 1.1 (`OFL.txt`) | Barlow Condensed SemiBold (github.com/jpt/barlow, Google Fonts); subset to Latin-1 with fontTools. Copyright 2017 The Barlow Project Authors (https://github.com/jpt/barlow) |
| `Spectral-SemiBold.ttf`, `SpectralSC-SemiBold.ttf` | SIL Open Font License 1.1 (`OFL.txt`) | Spectral SemiBold and Spectral SC SemiBold (github.com/productiontype/Spectral, Google Fonts); subset to Latin-1 with fontTools, and in Spectral SC each lowercase letter mapped straight to its small capital (the substitution its `ccmp` makes), so a renderer without that feature draws the small capitals. Copyright 2017 The Spectral Project Authors (https://github.com/productiontype/Spectral) |
| `Cinzel-Bold.ttf` | SIL Open Font License 1.1 (`OFL.txt`) | Cinzel Bold (github.com/NDISCOVER/Cinzel); unmodified. Copyright 2020 The Cinzel Project Authors (https://github.com/NDISCOVER/Cinzel) |
| `Newsreader-Medium.ttf` | SIL Open Font License 1.1 (`OFL.txt`) | Newsreader (github.com/productiontype/Newsreader); a static instance (wght 500, opsz 16) of the variable font, subset to Latin-1 with fontTools. Copyright 2020 The Newsreader Project Authors (http://github.com/productiontype/Newsreader) |
| `Cinzel[wght].ttf` | SIL Open Font License 1.1 (`OFL.txt`) | Cinzel, the variable font (wght 400–900) from google/fonts `ofl/cinzel`; unmodified. Copyright 2020 The Cinzel Project Authors (https://github.com/NDISCOVER/Cinzel) |
| `Newsreader[opsz,wght].ttf` | SIL Open Font License 1.1 (`OFL.txt`) | Newsreader, the variable font (wght 200–800, opsz 6–72) from google/fonts `ofl/newsreader`; unmodified. Copyright 2020 The Newsreader Project Authors (http://github.com/productiontype/Newsreader) |
| `ChakraPetch-Bold.ttf` | SIL Open Font License 1.1 (`OFL.txt`) | Chakra Petch Bold from google/fonts `ofl/chakrapetch`; unmodified. Copyright 2018 The Chakra Petch Project Authors (https://github.com/m4rc1e/Chakra-Petch.git) |
