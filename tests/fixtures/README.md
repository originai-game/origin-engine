# Test fixtures

Small projects built for one feature each. A fixture is not a template and
ships to nobody: it exists so a test can drive one engine feature on the
real stack in a few seconds. Each has a README (what it exercises, which
tests read it), generated or `.ogt` assets only, one or two reference frames
at most and `og.assert` invariants for its behaviour. Budget: under 10 s
warm per fixture run.

`scripts/check.sh fixtures` runs `og test` in each fixture whose inputs
changed since it last passed (below); `--all` and wave-end run all of them.
It also creates an empty project from `packages/create/template` under the
gate's evidence directory and runs its smoke test. That check includes
`packages/create` in its inputs and never builds into the shipped skeleton.

| Fixture | Feature | Tests that read it | Inputs (crates and packages it exercises) | Warm |
|---|---|---|---|---|
| [destruction-yard](destruction-yard/README.md) | destruction: fracture in the cook, breaking on impulses, dust, the cross-host world hash | its `tests/yard.og.ts`; `packages/engine/tests/destruction.test.ts`; `crates/og-runtime/tests/destruction.rs` | og-destruct, og-physics, og-cook, og-runtime, og-render, packages/engine | 3.5–6 s + 3.7 s cook |
| [vehicle-terrain](vehicle-terrain/README.md) | og.Vehicle on terrain, HUD, eased camera, particles, decals, engine audio | its `tests/drive.og.ts`; `scripts/check.sh web-reference`; `packages/engine/tests/presentation.test.ts` | og-physics, og-render, og-ui, og-audio, og-cook, og-runtime, packages/engine, templates/styles/scifi | 4.4–5.2 s |
| [vfx-hit](vfx-hit/README.md) | composed effects (type vfx): a hit and a heal, their events, one frame | its `tests/hit.og.ts` | og-runtime, og-render, og-cook, og-bundle, packages/engine | 0.9 s |
| [side-scroller](side-scroller/README.md) | 2D: flipbooks keyed from GIFs, a sprite animation graph, pixel terrain blown up from a system, 2D lights with soft shadows, parallax tilemap, Follow2D, vfx2d pixel effects | its `tests/slash.og.ts` | og-anim, og-cook, og-destruct, og-runtime, og-render, og-tilemap, packages/engine | 2.4 s + 5.7 s cook |
| [top-down](top-down/README.md) | 2D: a tileset and map, a GIF walk, loose PNGs packed into one texture (`crop each`), y-sorted sprites, warm 2D lamps and occluders; the atlas, penumbra, light-pass and sprite-count measurements | its `tests/walk.og.ts` | og-anim, og-cook, og-tilemap, og-runtime, og-render, packages/engine | 1.5 s + 0.4 s cook |
| [lockstep-room](lockstep-room/README.md) | two seats submit distinct inputs and share the same world | its `tests/inputs.og.ts`; packages/engine rooms and web-rooms tests | og-net, og-runtime, og-cli, packages/engine | 1.48 s |
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

The inputs of a fixture, as the `fixtures` step keys its cache: the
fixture's own directory, the native closure of `og` (`og_crate_dirs -p
og-cli`), `packages/engine` and the style packs and looks it reads.
