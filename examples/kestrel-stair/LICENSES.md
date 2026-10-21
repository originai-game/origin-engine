# Asset licences

Every asset of this game is made by code here (`tools/make-assets.ts` and
the modules it imports: deterministic, seeded, nothing downloaded),
written by hand, synthesised by `og cook` from `og sfx` presets, or one of
the engine's own CC0 / OFL assets. The one third-party file is the
Japanese font's source, fetched by `tools/fetch-font.ts` and not committed;
the subset cut from it is.

| File | Licence | Origin |
|---|---|---|
| `assets/models/stair.glb` | CC0 1.0 | self-made: the level built box by box and textured by `tools/level.ts` and `tools/make-assets.ts` from `src/layout.ts` |
| `assets/anims/humanoid.glb`, `assets/models/courier.glb`, `imp.glb`, `warden.glb`, `courier.glb.ogt`, `assets/rigs/warden.ogt` | CC0 1.0 | self-made: the clip library, the three box-built skinned bodies and their skeletons, `tools/rig.ts` |
| `assets/nav/stair.ogt`, `assets/ai/*.ogt` | CC0 1.0 | self-made: the navmesh file and the foes' behaviours, `tools/level.ts` |
| `assets/textures/*.png` | CC0 1.0 | self-made: the cape's and banners' cloth, the particles' spark, flame and dust, `tools/make-assets.ts` |
| `assets/env/dusk.hdr`, `dusk.ogt` | CC0 1.0 | self-made: a 64 x 32 synthetic sky, `tools/make-assets.ts` |
| `assets/fx/*.ogt` | CC0 1.0 | self-made: the particle emitters, `tools/make-assets.ts` |
| `assets/audio/*.ogt` | CC0 1.0 | `og sfx` presets (seed, pitch, length, volume chosen here) that `og cook` synthesises |
| `assets/audio/*.wav` | CC0 1.0 | self-made: the braziers' crackle, the crypt's drip, the imps' chitter and the bell, synthesised by `tools/make-assets.ts` |
| `assets/music/stair.wav` (not committed) | CC0 1.0 | self-made: "The Kestrel Stair", synthesised by `tools/make-assets.ts` |
| `assets/looks/*.ogt`, `assets/strings/*.ogt`, `assets/scenes/*.ogt`, every other `.ogt` | CC0 1.0 | self-made, written by hand (or by the tools above) |
| `assets/fonts/NotoSansJP-kestrel.otf`, `.otf.ogt` | SIL Open Font License 1.1 (`assets/fonts/OFL.txt`) | Noto Sans JP Regular (notofonts/noto-cjk, `Sans/SubsetOTF/JP/NotoSansJP-Regular.otf`, SHA-256 `dff723ba…5073`), subset to the CJK characters of `assets/strings/ja.ogt` by `tools/subset-font.py` (the engine's `templates/ui-kit/tools/subset-font.py`, copied) |
| `tools/fonts/NotoSansJP-Regular.otf` (not committed) | SIL Open Font License 1.1 | the subset's source, fetched by `tools/fetch-font.ts` |
| `assets/skins/kestrel/*` | CC0 1.0 art, SIL OFL 1.1 fonts (Instrument Sans, Newsreader, PT Mono) | a copy of the engine's paper style pack (`templates/ui-kit/assets/skins/paper`), retinted by `tools/make-assets.ts`; `assets/skins/kestrel/LICENSES.md` |
| `references/*/*.png` | CC0 1.0 | frames of this game recorded by `tests/frames.og.ts` |
