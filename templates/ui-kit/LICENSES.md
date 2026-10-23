# UI kit demo asset licences

| File | Licence | Origin |
|---|---|---|
| `assets/textures/icons.png` | CC0 1.0 | self-made by the Origin Engine authors: 32 white glyphs in an 8×4 grid, drawn by `tools/make-assets.py` (deterministic); the screens draw the iconoir set now, the grid only if it fails to load |
| `assets/icons/iconoir/*.svg` | MIT (`assets/icons/iconoir/LICENSE`, Copyright (c) 2021 Luca Burgio) | 72 icons of iconoir 7.12.1 (the platform's icon set, `iconoir` on npm), copied unchanged; `assets/icons/iconoir.ogt` (CC0) names them (and the kit's own names for 17 of them) for the cook, which renders them into a distance-field atlas |
| `assets/textures/backdrop.png` | CC0 1.0 | self-made: a dusk landscape behind the HUD screens, same script |
| `assets/fonts/NotoSansSC-Regular-subset.ttf` | SIL Open Font License 1.1 (`assets/fonts/OFL.txt`) | a byte-identical copy of `templates/game/assets/fonts/` (the platform's project-template font) |
| `assets/fonts/NotoSansJP-subset.otf`, `assets/fonts/NotoSansKR-subset.otf` | SIL Open Font License 1.1 (`assets/fonts/OFL.txt`) | Noto Sans CJK Regular (notofonts/noto-cjk; the TTC's Japanese and Korean faces) subset to the string tables' characters and ASCII by `tools/make-locale-fonts.py` |
| `assets/fonts/NotoSansArabic-subset.ttf` | SIL Open Font License 1.1 (`assets/fonts/OFL.txt`) | Noto Sans Arabic (notofonts/arabic) subset the same way, its layout features kept for shaping |
| `assets/fonts/Inter-Display.ttf` | SIL Open Font License 1.1 (`assets/fonts/OFL-Inter.txt`) | Inter Variable 5.3.0 (@fontsource-variable/inter, latin), instanced at opsz 32 and wght 600 and subset to Latin-1 and punctuation by `tools/make-faces.py`: the kit's display face |
| `assets/fonts/Nunito-Rounded.ttf` | SIL Open Font License 1.1 (`assets/fonts/OFL-Nunito.txt`) | Nunito Variable (@fontsource-variable/nunito, latin) instanced at wght 700 and subset the same way: the kit's rounded face (levels, scores) |
| `assets/fonts/NotoSansHebrew-subset.ttf`, `assets/fonts/NotoSansDevanagari-subset.ttf` | SIL Open Font License 1.1 (`assets/fonts/OFL.txt`) | Noto Sans Hebrew and Noto Sans Devanagari (notofonts; Godot 4.7's thirdparty copies) subset to the text screen's strings by `tools/make-faces.py`, layout features kept for shaping |
| `assets/strings/*.ogt` | CC0 1.0 | self-written string tables for the five locales |
| `screens/*.png` | CC0 1.0 | downscaled from `crates/og-ui/references/metal/kit-*.png`, which these playtests render |
| `../styles/<style>/` (pixel, cartoon, fantasy, scifi, paper, neon) | CC0 1.0 art and style files; SIL OFL 1.1 fonts | atlases, icons and `style.ogt` drawn by `tools/make-skins.py` (deterministic); fonts and their origins in each style's `LICENSES.md` and `OFL.txt` |
| `assets/audio/ui-*.wav` | CC0 1.0 | self-made: short synthesized blips for the UI sound hooks, `tools/make-skins.py` |
