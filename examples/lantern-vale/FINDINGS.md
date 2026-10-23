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
6. **Copying a style pack means rewriting it, and its fallback font is outside it** — annoyance. og-taste §1 says copy the pack, but `style.ogt` holds absolute ids (`styles/fantasy/...`) and its `fallback` names a 2.5 MB font not in the pack. Paths relative to the style file would fix it.
7. **No string field type** — annoyance. A text field's value (the player's name) cannot be world state. `templates/ui-kit/src/main.ts:376` keeps it in a module variable, against og-core §4 and og-ui §1. Workaround: twelve `u32` code points in `Prefs` (`nameOf`/`setName` in src/ui.ts). **Fixed:** a `str(capacity)` field type (og-core §4); `Prefs.name` is a `str(48)` and the ui-kit's text values live in its `Typed` resource.
8. **project.ogt's `tick`, `seed`, `scenes` and `scene` are not read** — annoyance. og-game §2 lists them; `og` reads only `entry` and `publish` (`og-cli/src/project.rs:82`, `publish.rs`). The values that apply are `GameDesc`'s, and the two can drift apart silently.
9. **BLOCKER: `og.load.textureArray` is not a function.** It is in `types/index.d.ts`, og-game §8, wiki/load.md and both hosts (`og_load_texture_array`), but missing from `packages/engine/src/runtime.ts`'s `load` object (~line 970). Repro: a game whose `start` calls `og.load.textureArray(w, "a.ogt")`; `OG_ENGINE_PACKAGE=packages/engine og run <it> --headless --commands "0.1:quit"` prints `TypeError: og.load.textureArray is not a function`. It typechecks, and the knowledge check passes. Workaround: an `array true` flipbook (`assets/sprites/faces-sheet.ogt`, using `FlipbookInfo.array`). `assets/sprites/faces.ogt` is kept as the repro.
10. **A throw in `GameDesc.start` is only a WARN** — annoyance. The run continues on a half-built world, and `og run` exits 0 (the #9 repro exits 0).
11. **`og.ui.configure` replaces the whole UiDesc** — annoyance. Configuring sounds (og-audio §3) or `reducedMotion` after `GameDesc.look` drops the look's style; with `font: null`, which draws nothing, every glyph vanishes. No warning, and the configure docs don't say it replaces. Workaround: `configureUi` in src/ui.ts always passes `og.render.look(world)?.style`, the sounds and reduced motion together. Repro: drop `style` there, then `og run examples/lantern-vale --headless --commands "1:screenshot:t.png;1.1:quit"`.
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

## After wave 7

Each finding's state on main at 767ecf84 (the streams' sections in `docs/phase-1.md`; owed rows are that file's).

1. Fixed: api-runtime, c364d491 (og-audio names `AudioSource.loop`; the knowledge check reads `key: value` spans). No workaround here.
2. Fixed: cook-vox, 6e5a369b (`og.anim.clip(info)` finds a model's clip, og-game §9 shows it). The game's `og.load.animation(...).clips.find` still works and stays.
3. Fixed: cook-vox, 6e5a369b (`GltfInfo.animations` lists the model's clips). No workaround here.
4. Fixed: cook-vox, 6e5a369b (`voxScale`, `voxOrigin`, `voxFps`, `voxClip` in the `og.load.gltf` doc, og-game §5 and og-build §7). No workaround here.
5. Fixed: cook-vox, 42e622cb (MagicaVoxel frames cook to skinned mesh-swap clips). The slime stays a glTF from `tools/make-assets.ts`: it bends between two weighted joints and is glassy, which a `.vox`'s whole-frame swaps and one rough material do not give. Open: `nTRN` key frames and the cost of a folded frame (row 63).
6. Fixed: ui-text, 9d00fb21 (style ids relative to the style file). The game's copy keeps its absolute ids, which still work. Open by decision: the CJK fallback stays outside the pack (row 66).
7. Fixed: reflect-string, 294888a2. `Prefs.name` is a `str(48)`; the twelve `u32` code points and `nameOf` are gone.
8. Fixed: api-runtime, 457e659a (project.ogt's `tick`, `seed`, `scenes` and `scene` are read; set in both places to different values they refuse the engine). The game's `GameDesc` and project.ogt agree (tick 60, seed 7).
9. Fixed: api-runtime, 0e3419b0 (`og.load.textureArray` and `EngineProbe.listen` on the runtime; `api-surface.test.ts` walks every IDL call). The `textureArray` call is restored and the flipbook-array sheet removed (3eeafa8a). Open: runtime members the IDL does not declare are unchecked (row 64).
10. Fixed: api-runtime, 5dfc5a7b (a throw in `start` fails the run; `og run`, `og test` and `og dev` exit non-zero). Open: no browser test of the shell's crash screen (row 64).
11. Fixed: ui-text, 34d86c21 (`og.ui.configure` keeps the fields it is not given and warns when no font is left). `configureUi` no longer re-reads the look's style: it passes only the sounds and reduced motion.
12. Fixed: api-runtime, 38472c71 (`set:` takes `true`/`false`).
13. Fixed: api-runtime, 38472c71 (more unnamed `get:` steps on one tick write `get-<tick>-2.json`, ...).
14. Fixed: api-runtime, 38472c71 (`FieldOptions` says which writes are refused outside min..max and which are stored as given; `set:` names the range).
15. Fixed: render-style, 7e3ae322 (a material's outline draws silhouettes and contact lines, never the edges between its own faces). The look's `outlineWidth 0` removed at 767ecf84: the anime cel preset's outlines draw.
16. Fixed: render-style, 51e765e9 (`StandardMaterialDesc.styled`; a look's toon passes by a material with a physical lobe). The look's `toon false` and `styleModels`' per-model toon removed at 767ecf84; `styleModels` keeps the lanterns' clear coat. The dawn look now names `toon false` itself, by choice.
17. Fixed: render-style, 46c4d3ea (`Sprite.toneMapped`). The bubbles' 0.6 dimming removed at 767ecf84: they are `toneMapped: false`.
18. Fixed: render-style, 8ce76910 (og-taste §8: a warm dusk with shadows is 16.5–17.25 h; DayNight was right). No change here: the run spans 15:30–19:30.
19. Fixed: render-style, b9c3ed87 (flatten 0 is a real tint, the bands step through hue, og-taste §12 says bands are not for top-down). The game keeps bands for the results' high view; its results frame re-recorded at 767ecf84.
20. Fixed: api-runtime, 38472c71 (`get:og.Time` answers every field, rate and the time scale; og.Time's doc names `og.TimeScale`).
21. Fixed: ui-text, 8a288955 (`type:` keeps its text to the `;`; bare keys typing only until the first `og.TextInput` is documented). The settings playtest still types `Mei` and commits `小灯` through an input method.
22. Fixed: ui-text, 8a288955 (og-ui §7 holds the one mixed-script rule, §3 points to it). No workaround here.
23. Fixed: ui-text, dbcdf1ad (a field is named by its label, described by its placeholder). No workaround here.
24. Fixed: ui-text, 7d982be7 (`ResultsDesc.bestLabel`, `newBestLabel`, `shareLabel`). The game passes `results.best` and `results.newBest` from both tables; the locales playtest checks them.
25. Fixed: ui-text, 213ed08f (og polish leaves out text mid-transition; og-build §5 says why thin CJK strokes measure lower).
26. No defect (6c804499 corrected it to a nit): the dev profile optimises the heavy crates per package and the gate measures on it by rule; no row.
27. Fixed: cook-vox, 42e622cb (a graph-less `.vox` is centred like a graph's). The scene graph is removed from `voxBytes` and the files regenerated (c962b1e6, byte-identical blobs).
28. Fixed: api-runtime, c364d491 (og-game §13 says `og.Velocity` is optional and where `world.set` refuses).
29. Fixed: ui-text, ddde1e74 (every pack's placeholder at 4.70:1 or better, the focus ring cut under controls, the packs' test covers fields). The game's own darkened copy (2f924b3e) is unchanged.
30. Fixed: api-runtime, 38472c71 (`get` answers the light og.DayNight drives as it is drawn).
31. Fixed: api-runtime, 38472c71 (og-feel reads values with `get:`).
32. Fixed: ui-text, 8b87652b (it was the screenshot's PNG deflate in the dev profile, counted as step time: the capture is left out of the step and miniz_oxide is optimised in dev; frame max 948.7 to 20.0 ms in the locales playtest).
