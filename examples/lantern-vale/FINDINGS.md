# What Lantern Vale found

Every problem met writing this game from the knowledge package (skills and wiki) as a game author would, in the order met. Nothing in the engine was changed; workarounds are named. Ratings: **blocker** (the documented way does not work), **annoyance** (costs real time, or silently wrong), **nit** (small, cosmetic, docs only). Engine 721886b7 then main 2133be1c; dev-profile `og` and addon; tier metal. Commands run from the repository root, `og` = `target/debug/og`.

| | Blocker | Annoyance | Nit | Total |
|---|---|---|---|---|
| Count | 1 | 15 | 16 | 32 |

Top five: #9, #16, #15, #11, #27 (with #4).

1. **og-audio §5 says `looping: true`; the field is `AudioSource.loop`** — nit. TypeScript catches it in an object literal; the knowledge name check misses it (not written `Type.field`).
2. **`og.anim.clip` only finds flipbook clips** — nit. A skinned model's clips are found with `og.load.animation(...).clips.find(c => c.name === "walk")`; og-game §9 never shows it.
3. **`GltfInfo.animations` is documented as never filled** ("the cook ... does not cook them yet") — nit. It is `[]` for models/slime.glb, which has `idle` and `chase`.
4. **`.vox` import settings are undocumented; og-game §5 says "one voxel one metre"** — annoyance. The cook takes `model { voxScale voxOrigin bottom }` (`og-cook/src/cook/vox.rs` `VoxSettings`); no skill or wiki page names either. Found by reading the cook.
5. **A `.vox` model cannot be animated** — annoyance. It cooks to a static glTF (no skin, no clips), so a voxel slime on `AnimationPlayer.stepFps` is impossible. Workaround: tools/make-assets.ts meshes the slime's voxels into a skinned glTF itself.
6. **Copying a skin pack means rewriting it, and its fallback font is outside it** — annoyance. og-taste §1 says copy the pack, but `skin.ogt` holds absolute ids (`skins/fantasy/...`) and its `fallback` names a 2.5 MB font not in the pack. Paths relative to the skin file would fix it.
7. **No string field type** — annoyance. A text field's value (the player's name) cannot be world state. `templates/ui-kit/src/main.ts:376` keeps it in a module variable, against og-core §4 and og-ui §1. Workaround: twelve `u32` code points in `Prefs` (`nameOf`/`setName` in src/ui.ts).
8. **project.ogt's `tick`, `seed`, `scenes` and `scene` are not read** — annoyance. og-game §2 lists them; `og` reads only `entry` and `publish` (`og-cli/src/project.rs:82`, `publish.rs`). The values that apply are `GameDesc`'s, and the two can drift apart silently.
9. **BLOCKER: `og.load.textureArray` is not a function.** It is in `types/index.d.ts`, og-game §8, wiki/load.md and both hosts (`og_load_texture_array`), but missing from `packages/engine/src/runtime.ts`'s `load` object (~line 970). Repro: a game whose `start` calls `og.load.textureArray(w, "a.ogt")`; `OG_ENGINE_PACKAGE=packages/engine og run <it> --headless --commands "0.1:quit"` prints `TypeError: og.load.textureArray is not a function`. It typechecks, and the knowledge check passes. Workaround: an `array true` flipbook (`assets/sprites/faces-sheet.ogt`, using `FlipbookInfo.array`). `assets/sprites/faces.ogt` is kept as the repro.
10. **A throw in `GameDesc.start` is only a WARN** — annoyance. The run continues on a half-built world, and `og run` exits 0 (the #9 repro exits 0).
11. **`og.ui.configure` replaces the whole UiDesc** — annoyance. Configuring sounds (og-audio §3) or `reducedMotion` after `GameDesc.look` drops the look's skin; with `font: null`, which draws nothing, every glyph vanishes. No warning, and the configure docs don't say it replaces. Workaround: `configureUi` in src/ui.ts always passes `og.render.look(world)?.skin`, the sounds and reduced motion together. Repro: drop `skin` there, then `og run examples/lantern-vale --headless --commands "1:screenshot:t.png;1.1:quit"`.
12. **`set:` wants 0/1 for a boolean** — nit. `set:og.Stylise.ink=false` → "'false' is not a number".
13. **Several unnamed `get:` steps on one tick overwrite `get-<tick>.json`** — nit.
14. **`set:` refuses values outside a field's range** (`set:Tuning.cameraHeight=3`, min 4) while `FieldOptions` says "nothing clamps writes" — nit.
15. **`MaterialStyle.outlineWidth` buries `.vox` models in outline colour** — annoyance (bug). The `anime-cel-3d` hull outlines on unwelded flat quads draw shell fragments over the faces: tree canopies become a mesh of dark lines and the slime turns black. Repro: `og run examples/lantern-vale --headless --commands "0.5:run;0.6:warp:-20:2;0.6:set:Tuning.cameraHeight=4;0.6:set:Tuning.cameraBack=5;0.7:set:og.MaterialStyle.outlineWidth=2.5;1.5:screenshot:o.png;1.6:quit"`. Workaround: `outlineWidth 0`, with `Stylise.inkDepth` for silhouettes.
16. **A toon look switches every physical material off, with no opt-out** — annoyance (API gap). `MaterialStyle.toon` reaches every material that leaves `toon` at its default (false), and toon materials ignore clearcoat and transmission, so the lacquer and the jelly vanish silently (`set:og.MaterialStyle.toon=0` brings them back). Workaround: the look sets `toon false`; `styleModels` in src/world.ts makes the voxel models toon one by one.
17. **World sprites ride auto exposure and bloom** — nit. Cream pixel bubbles blow out to white at dusk. Workaround: `Sprite.color` 0.6.
18. **og-taste's "warm dusk at 17–18 h" has no sun shadows** — nit. At the default tilt shadows are gone by about 17:00 and the frame goes flat; the run uses 15:30–19:30.
19. **Depth bands are unusable top-down** — annoyance (docs). The play area falls in the near band (one flat colour; the hero vanishes). `depthBandsFlatten 0` ("a tint") still replaces each band's hue. The far colour shows only past `depthBandsFar` (ends blend through grey). Nothing says "not for top-down". Used only for the results' high view.
20. **`get:og.Time` answers only `{dt, elapsed, tick}`** — annoyance. `scale` and `hitStop` live in an undocumented `og.TimeScale`, and `scaledDt` and `rate` are not answered at all.
21. **`type:` trims a trailing space, and bare `key_down` doesn't type into a focused field natively** — nit. The field's doc says "else the keys". The playtest types `Mei`, then commits `小灯` through an input method.
22. **Mixed-script labels: og-ui §3 (per run) and §7 ("one label uses one font") disagree** — annoyance. A CJK subset carrying Latin-1, as the archetypes build theirs, draws the whole mixed label in Noto. A CJK-only subset (`tools/subset-font.py`) gives per-run fonts.
23. **A text field's accessible name is its placeholder, not its label** — nit.
24. **`og.ui.resultsScreen`'s "Best …" and "New best" are English only** — annoyance. No `ResultsDesc` field or table key for them (see locales test `results-zh.json`).
25. **og polish counts toasts under a dialog's scrim as low contrast** (1.15:1, 1.31:1), though og-build §5 excludes scrim-dimmed text — nit. Thin CJK strokes also measure lower: "设置" 3.12:1 where the same colours pass in Latin.
26. **Frame time is measured on the dev profile** — nit (the first reading of this was wrong: `Cargo.toml` does set `[profile.dev.package.<crate>] opt-level` for rapier, parry, glam, nalgebra, wgpu, naga, meshopt, basis-universal and every og-* crate; what dev lacks against release is LTO and one codegen unit). Numbers on metal, 1280×720:

    | Measure | Before the merge (loaded machine) | After |
    |---|---|---|
    | `og run --json`, 600 ticks of play | advanceMs 5700–6800 (9.5–11.4 ms/tick) | 3681 (6.1 ms/tick) |
    | Title, 600 ticks | 7900 | not measured |
    | Polish step | 3.9–4.8 ms median, p95 5.7–11.7 | 3.8–4.3 median, p95 4.9–11.1 |
    | Polish render | 5.9–7.8 ms median | 2.0–2.5 median |
    | Step + render p95 | 14.3–21.8 ms (score 56) | 7.9–13.7 ms (score 78) |

    Step p95 doubles in play against the title. Bloom and auto exposure off saves about 2–3 ms (noisy).
27. **A `.vox` without a scene graph is placed by its corner** — annoyance (bug). `vox.rs` places graph-less models "side by side along x" with each model's node at size/2, so a lone model spans 0..size. The lantern's wick hung 0.35 m off the cage, and trees and rocks sat half a footprint off their colliders. Workaround: every generated `.vox` gets a root/group/transform(`_t 0 0 0`)/shape graph (`voxBytes`). Repro: drop the graph chunks, then cook, then `og run examples/lantern-vale --headless --commands "0.3:run;0.4:set:Tuning.cameraHeight=4;0.4:set:Tuning.cameraBack=4;0.5:warp:-21:10.5;1.5:screenshot:wick.png;1.6:quit"`.
28. **`og.Velocity` is optional for a body**, though og-game §13 implies it is required, and `world.set(e, og.Velocity)` on a body without it refuses — nit.
29. **The fantasy pack's placeholder fails 4.5:1** (2.93:1), although og-taste says the packs are tested — nit. Even at the darkest ink the game's copy reaches only 3.93:1, because the placeholder is drawn dimmed.
30. **`get:sun.og.DirectionalLight` doesn't reflect the DayNight drive** (white, 2.4, unrotated at every hour) — nit.
31. **og-feel §13 asks for values from `dump_world`, which lists resources by name only** — nit. `get:` works (og.Camera, og.TimeScale).
32. **Switching to Chinese stalls a frame for 0.8–1.0 s** — annoyance (perf). Polish's frame max is 780–996 ms in exactly the three playtests that switch to zh-CN (frames, locales, settings) and 15–17 ms elsewhere; headless `advanceMs` shows no difference, so it is on the render side.

## What worked well

og cook: 47 assets cold in 8.2 s, incremental about 90 ms, no warnings. The access errors name the fix. Streaming `.vox`/texture groups, `applyLook` at a tick, chunked music, `og.physics.overlap` on a layer, the controller shoving bodies, a skinned glTF on twos and the flipbook array each worked the first time they were asked correctly. Two processes of a full session end on one hash. Every polish finding named its fixing section.
