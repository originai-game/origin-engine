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
