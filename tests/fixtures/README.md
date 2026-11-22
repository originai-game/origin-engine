# Test fixtures

Small projects built for one feature each. A fixture is not a template and
ships to nobody: it exists so a test can drive one engine feature on the
real stack in a few seconds. Each has a README (what it exercises, which
tests read it), generated or `.ogt` assets only, one or two reference frames
at most and `og.assert` invariants for its behaviour. Budget: under 10 s
warm per fixture run.

`scripts/check.sh fixtures` runs `og test` in each fixture whose inputs
changed since it last passed (below); `--all` runs all of them.
It also creates an empty project from `packages/create/template` under the
gate's evidence directory and runs its smoke test. That check includes
`packages/create` in its inputs and never builds into the shipped skeleton.

| Fixture | Feature | Tests that read it | Inputs (crates and packages it exercises) | Warm |
|---|---|---|---|---|
| [destruction-yard](destruction-yard/README.md) | destruction: fracture in the cook, breaking on impulses, dust, the cross-host world hash | its `tests/yard.og.ts`; `packages/engine/tests/destruction.test.ts`; `crates/og-runtime/tests/destruction.rs` | og-physics, og-cook, og-runtime, og-render, packages/engine | 3.5–6 s + 3.7 s cook |
| [vehicle-terrain](vehicle-terrain/README.md) | og.Vehicle on terrain, HUD, eased camera, particles, decals, engine audio | its `tests/drive.og.ts`; `scripts/check.sh web-reference`; `packages/engine/tests/presentation.test.ts` | og-physics, og-render, og-ui, og-audio, og-cook, og-runtime, packages/engine, templates/styles/scifi | 4.4–5.2 s |
| [vfx-hit](vfx-hit/README.md) | composed effects (type vfx): a hit and a heal, their events, one frame | its `tests/hit.og.ts` | og-runtime, og-render, og-cook, og-bundle, packages/engine | 0.9 s |
| [anim-markers](anim-markers/README.md) | og.AnimationEvent's state, layer and dominant in a cross-fade and a three-way blend | its `tests/markers.og.ts`; og-anim `tests/graph.rs`, `tests/playback.rs`; `packages/engine/tests/anim.test.ts` | og-anim, og-runtime, og-cook, packages/engine | 0.7 s |
| [hit-volumes](hit-volumes/README.md) | og.play: a hit volume on a posed hand swept through thin dummies and a wall between ticks, once a swing; og.play.candidates with sight | its `tests/swing.og.ts`; `packages/engine/tests/play.test.ts` | og-play, og-runtime, og-anim, og-physics, packages/engine | 1.3 s |
| [combat-arena](combat-arena/README.md) | og.systems.combat: a light-light-heavy combo chained by buffered presses in windows into a dummy, i-frames, Died once, a dodge only in the cancel window | its `tests/combo.og.ts`; `packages/engine/tests/combat.test.ts` | og-runtime, og-play, og-input, og-physics, packages/engine | 1.2 s |
| [interact-house](interact-house/README.md) | og.systems.interact: focus by rule and sight, a 30-tick hold that opens a door and its navmesh link for a guard, a lever, a crate carried and thrown onto a pressure plate that opens a gate | its `tests/house.og.ts`; `packages/engine/tests/interact.test.ts` | og-runtime, og-play, og-input, og-physics, og-nav, packages/engine | 1.5 s |
| [side-scroller](side-scroller/README.md) | 2D: flipbooks keyed from GIFs, a sprite animation graph, pixel terrain blown up from a system, 2D lights with soft shadows, parallax tilemap, Follow2D, vfx2d pixel effects | its `tests/slash.og.ts` | og-anim, og-cook, og-physics, og-runtime, og-render, og-tilemap, packages/engine | 2.4 s + 5.7 s cook |
| [top-down](top-down/README.md) | 2D: a tileset and map, a GIF walk, loose PNGs packed into one texture (`crop each`), y-sorted sprites, trees swaying in og.Wind (og.Sprite.sway), warm 2D lamps and occluders; the atlas, penumbra, light-pass and sprite-count measurements | its `tests/walk.og.ts`, `tests/sway.og.ts` | og-anim, og-cook, og-tilemap, og-runtime, og-render, packages/engine | 1.5 s + 0.4 s cook |
| [lockstep-room](lockstep-room/README.md) | two seats submit distinct inputs and share the same world | its `tests/inputs.og.ts`; packages/engine rooms and web-rooms tests | og-net, og-runtime, og-cli, packages/engine | 1.48 s |
| [input-timing](input-timing/README.md) | ActionState.pressedAt from the press's time in its tick; ActionDesc.bufferTicks and og.input.consume | its `tests/timing.og.ts`; og-input `tests/actions.rs` | og-input, og-probe, og-runtime, packages/engine | 0.7 s |
| [data-tables](data-tables/README.md) | og.table: two tables naming each other's rows read as components, held by handle; a reload, a refused row by its line, a deleted row og.table.missing names | its `tests/tables.og.ts`; `packages/engine/tests/table.test.ts`; og-cook `cook::systems::table` | og-core, og-cook, og-bundle, og-runtime, packages/engine | 0.6 s |
| [inventory-chest](inventory-chest/README.md) | og.systems.inventory: pickups taken through sensors into a bag, all-or-nothing moves, split and merge, a craft that needs room, loot on og.Rng | its `tests/chest.og.ts`; `packages/engine/tests/inventory.test.ts` | og-runtime, og-physics, packages/engine | 1.2 s |
| [rive](rive/README.md) | Rive import: artboards, paths, paints, keyframes, nested artboards, the import settings | og-cook `riv::tests`, `tests/rive.rs`; og-tests-cooked `ui_rive_*` | og-cook, og-ui, og-render | 0.2 s |
| [pixel-font](pixel-font/README.md) | pixel-font grid diagnostics (asset-only fixture) | og-cook `a_pixel_font_cooks_on_its_grid` | og-cook, templates/styles/cjk | 0.25 s |
| [cook-assets](cook-assets/README.md) | model, texture and audio loading | cook/runtime loader tests; engine animation/Basis tests | og-cook, og-runtime | owning tests |
| [ai-nav](ai-nav/README.md) | authored navmesh and guard machine | og-ai tests | og-ai | owning tests |
| [motion](motion/README.md) | animation compression and retargeting | cook retarget; runtime motion/LOD | og-cook, og-runtime | owning tests |
| [motion-feet](motion-feet/README.md) | og.motion's drawn feet: planted, stepped, never sliding, over walks, a run, jumps, a turn and held poses; its poses at fixed ticks on the web | its `tests/feet.og.ts`; `packages/engine/tests/presentation.test.ts`; `packages/engine/tests/web/motion-ticks.ts` | og-motion, og-runtime, og-anim, og-render, packages/engine | 5–7 s with its cook |
| [baked-light](baked-light/README.md) | baked-scene binding | runtime baked_scenes | og-runtime | owning tests |
| [texture-arrays](texture-arrays/README.md) | material arrays and streaming | runtime arrays/release; cook stream | og-cook, og-runtime | owning tests |
| [sprite-2d](sprite-2d/README.md) | sprite arrays and 2D web export | runtime sprite arrays/release; web export | og-runtime, packages/engine | owning tests |
| [ui-text](ui-text/README.md) | fonts, shaping, icons and locale tables | UI/cook text tests, engine UI tests | og-ui, og-cook, packages/engine | owning tests |
| [terrain-stream](terrain-stream/README.md) | resident/streamed height samples agree | engine terrain-stream test | og-runtime, packages/engine | owning test |
| [browser-host](browser-host/README.md) | Player store and reflected probe contracts | web player/wire/export tests | packages/engine, og-runtime | owning tests |
| [vfx-release](vfx-release/README.md) | downloadable effect packs: a pack, its release archive, signed (not a project) | og-cli `vfx_add_downloads_a_signed_pack_and_refuses_anything_else`; `scripts/release-dist.test.ts`; install test step 10 | og-cli, packages/engine, scripts/release-dist.ts | owning tests |
| [sensors](sensors/README.md) | og.physics.sensorContents: a pressure plate's contents every tick against the crates over it and its events | its `tests/plate.og.ts`; og-physics `tests/sensors.rs`; `packages/engine/tests/hosts.test.ts` (the physics scene's sensor) | og-physics, og-runtime, packages/engine | 0.5 s |

The inputs of a fixture, as the `fixtures` step keys its cache: the
fixture's own directory, the native closure of `og` (`og_crate_dirs -p
og-cli`), `packages/engine` and the style packs and looks it reads.
