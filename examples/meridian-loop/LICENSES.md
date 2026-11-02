# Asset licences

Every asset of this game is made by code here (`tools/make-assets.ts` and
the modules it imports: deterministic, seeded, nothing downloaded) or
written by hand, except the UI style's art and fonts, which are the
engine's former neon style pack, copied once and committed.

| File | Licence | Origin |
|---|---|---|
| `assets/models/track.glb`, `scenery.glb`, `ground.glb` | CC0 1.0 | self-made: the circuit's road and walls, the embankments, tubes, gantry, gates, skyline and ridges, and the ground grid, built from `src/track.ts` by `tools/circuit.ts` |
| `assets/models/car.glb`, `ghost.glb`, `wheel.glb`, `ghost-wheel.glb` | CC0 1.0 | self-made: the car, its ghost and the wheels, `tools/car.ts` |
| `assets/textures/*.png` | CC0 1.0 | self-made: the mini-map, the smoke puff, the spark and the skid streak, `tools/make-assets.ts` and `tools/circuit.ts` |
| `assets/audio/*.wav` | CC0 1.0 | self-made: the engine and tyre loops, the knocks, landings, cues and UI sounds, synthesised by `tools/audio.ts` |
| `assets/music/meridian.wav` (not committed) | CC0 1.0 | self-made: "Meridian", synthesised by `tools/audio.ts` |
| `assets/fx/*.ogt`, `assets/looks/*.ogt` | CC0 1.0 | self-made: the emitters and the look, `tools/make-assets.ts` |
| `assets/strings/*.ogt`, `assets/scenes/*.ogt`, every other `.ogt` | CC0 1.0 | self-made, written by hand |
| `assets/styles/meridian/*` | CC0 1.0 art, SIL OFL 1.1 fonts (Chakra Petch, Rajdhani) | a copy of the engine's former neon style pack (folded into cyber, 2026-11-02), its `style.ogt` retinted; `assets/styles/meridian/LICENSES.md` |
| `references/*/*.png` | CC0 1.0 | frames of this game recorded by `tests/frames.og.ts` and `tests/published.og.ts` |
