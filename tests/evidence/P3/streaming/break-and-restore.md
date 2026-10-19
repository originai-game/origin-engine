# Phase 3 streaming round: break and restore

Every new test was broken once in the code it covers and failed, then the
code was restored and it passed. Each row is the edit made and what the
test read when it failed (2026-10-19 and after, Apple M4 Pro, Metal unless
the row says otherwise). Each break was applied to one source file, the
named test run, and the file put back.

## Terrain placement (`crates/og-tests/tests/terrain.rs`)

| Broken | Edit | Failed |
|---|---|---|
| the terrain's transform | `terrain.wgsl`: `terrain_world` returns the local point plus the translation (rotation and scale ignored) | `a_terrain_honours_its_transform`: "turned along z: (0, 0, 24) ... is [255, 0, 255]" (the turned strip still along x) |
| sample in the terrain's space | `og-runtime/src/terrain.rs` `local()`: the world point minus the translation (the turn and scale not undone) | `packages/engine/tests/terrain.test.ts` "a turned and scaled terrain ...": `along.inside` received false (60 m along world -z is off an unturned hill) |

## The tile pyramid (`crates/og-tests/tests/terrain.rs`, `crates/og-render/src/terrain_tiles.rs`, `terrain.rs`)

| Broken | Edit | Failed |
|---|---|---|
| the capacity | `terrain_tiles_wanted`: `capacity.max(1000)` instead of `capacity.max(1)` | `wanted_tiles_are_ranked_and_closed_under_their_parents`: 85 tiles where 6 were asked for |
| the fade | `sync`: a fading tile's fade never advances | `a_streamed_terrain_draws_what_has_arrived`: "streamed 0.0293 against the field in memory" (the arrived tiles never show) |

## The web host's inflater (`packages/engine/tests/stream.test.ts`)

| Broken | Edit | Failed |
|---|---|---|
| zlib, not raw deflate | `worker.ts` `installInflater`: `DecompressionStream("deflate-raw")` | "the Worker's inflater answers each zlib stream ...": key 7 received nothing where 34,322 bytes were expected |

## Scattered copies from chunks (`crates/og-tests/tests/terrain.rs`)

| Broken | Edit | Failed |
|---|---|---|
| the dithered fade | `scatter_common.wgsl` `scatter_dropped`: never drops a pixel | `scattered_copies_are_drawn_from_chunks_and_fade_by_dither`: "a copy 70.5 m out is filled 1.00" |

`terrain-grass` (metal) was re-recorded for this change: the 36,000 tufts
now thin out by a dither over the outer third of their 60 m instead of
shrinking, and are shaded by the scatter shader (`terrain-grass-before-after.png`:
before left, after right; the bottom row the meadow at full size).
`scatter_follows_its_layer_distance_and_stays_put` now asserts the copies
near the edge keep their full size where it asserted they had shrunk
(the behaviour asked for changed); every other assertion is unchanged.

## Terrain edits as world state (`packages/engine/tests/terrain.test.ts`)

| Broken | Edit | Failed |
|---|---|---|
| the replay | `og-runtime/src/terrain.rs` `TerrainBlock::apply`: `log.replay = false` (the restored log kept, the ground not rebuilt) | "terrain edits are world state ...": after restoring the raised snapshot, sample expected 6.10 and received 1.10 (the later edits stayed) |

## Streaming groups through the cook (`crates/og-cook/tests/stream.rs`)

| Broken | Edit | Failed |
|---|---|---|
| the proxy's group | `og-cook/src/cook/mod.rs` `output_group`: only `#anim` kept out of its asset's group (a `#proxy` packed with what it stands in for) | `groups_are_packed_listed_and_stood_in_for`: `models/house-a.glb#proxy` left `Some("village")`, right `None` |

## Streamed resources in place: textures and sounds

| Broken | Edit | Failed |
|---|---|---|
| a replaced texture | `og-render/src/lib.rs` `sync`: materials rebuilt only when their own revision changed, and the sprite and trail groups of a replaced texture kept | `a_replaced_texture_reaches_the_sprites_that_draw_it` ("the replacement is drawn": left `[255, 0, 0]`, right `[0, 0, 255]`) and `a_replaced_texture_reaches_the_materials_that_sample_it` (still red) |
| a silent stand-in's handoff | `og-audio/src/sound.rs` `Reader::frame`: the stand-in's cell never read (`cell.get().filter(\|_\| false)`) | `a_silent_stand_in_becomes_its_sound_in_the_voice_already_playing`: "the playing voice continues with the samples" (0.0) |

## Streaming groups at run time (`packages/engine/tests/asset-stream.test.ts`)

| Broken | Edit | Failed |
|---|---|---|
| a resident model's materials | `og-runtime/src/stream.rs` `upgrade`: the full model's materials never set (`set_material` skipped), so the proxy's untextured materials stay | `three groups load and unload along the path...`: the frames at z -40, -150 and -230 read `grey` where `red`, `green` and `blue` were expected |
| the publish split | `packages/engine/src/node/web-build.ts`: the first group's precache list no longer leaves out the packs (`&& !packs.has(f)` removed) | `og build --publish ships each group as its pack beside the first group...`: the OG_SW precached files equal the first group (the three packs were in it) |

## Streaming groups in Chrome (`packages/engine/tests/web/stream.ts`)

| Broken | Edit | Failed |
|---|---|---|
| the service worker's lazy cache | `packages/engine/shell/sw.ts` fetch: a fetched pack never put in the cache (`cache.put` removed) | "the service worker keeps every pack the engine fetched" (no pack cached) and "offline, every group the player has been to loads from the service worker's cache" (every zone `grey`, its proxy) |
