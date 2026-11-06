# Asset licences

Nothing in this project is downloaded: every asset is self-made by the
Origin Engine authors or one of the engine's own CC0 assets. The circuit,
the cars, the barriers, the grandstand, the gantry, the trees and the
hills are not files at all: `src/track.ts` and `src/mesh.ts` build them as
vertex-coloured meshes in `start`.

| File | Licence | Origin |
|---|---|---|
| `src/track.ts`, `src/mesh.ts` (the circuit and every model) | CC0 1.0 | self-made: a Catmull-Rom circuit and flat-shaded meshes generated in code |
| `assets/audio/*.wav` | CC0 1.0 | self-made: the engine and tyre loops, the contacts, the countdown, lap, sector, best-lap, boost and finish cues and "Redline", a 132 bpm loop, synthesised byte for byte by `tools/make-assets.ts` (the engine, tyre, contact and cue voices after `examples/meridian-loop`'s `tools/audio.ts`) |
| `assets/textures/puff.png`, `puff.png.ogt`, `spark.png` | CC0 1.0 | the engine's: `examples/meridian-loop` (`tools/make-assets.ts`) |
| `assets/fx/smoke.ogt`, `sparks.ogt` | CC0 1.0 | the engine's: `examples/meridian-loop`'s tyre smoke and contact sparks, retuned by hand |
| `assets/ui/main-menu.ogt`, `pause.ogt` | MIT | the engine's screen documents (`og ui eject`), edited for this game |
| `assets/ui/racing/race-results.ogt` | MIT | the racing kit's results document (`packages/engine/extensions/racing/race-results.ogt`), edited for this game |
| `assets/strings/*.ogt` | CC0 1.0 | self-made: the English and Chinese string tables |
| `assets/styles/redline/*` | engine licence (MIT) for the pack, SIL OFL 1.1 fonts | a copy of the engine's broadcast style pack (`templates/styles/broadcast`); its fonts and icons stay in the engine's built-in packs, `assets/styles/redline/LICENSES.md` |
| `references/*/*.png` | CC0 1.0 | frames of this game recorded by `tests/frames.og.ts` and `tests/frames-phone.og.ts` |
