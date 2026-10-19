# M4 glTF edit to reload, M5 timed 8-frame sweep (owed row 34)

Apple M4 Pro (14 cores), macOS, 2026-10-19, `target/release/og` and the
release `og-host-node` addon at 3d7ad18. The machine ran four other agents'
gates and builds throughout (1-minute load average 51 to 87).

The project is the 3D template scaffolded by `packages/create` (its
`assets/scenes/main.ogt` spawns `dropship` with `model "models/dropship.glb"`).

## M4: a glTF edit reloads in at most 300 ms with the change shown

```
export OG_ENGINE_PACKAGE=packages/engine OG_NATIVE_ADDON=<copy of target/release/libog_host_node.dylib named origin-engine.node>
og dev <project> --port 47913 --exit-after 8 --json
```

A driver waited for the probe, captured a frame (`og probe capture`), then
eight times rewrote `assets/models/dropship.glb` in place, swapping the
`ShipHull` material's `baseColorFactor` between the shipped grey
`[0.140…,0.145…,0.150…,1]` and red `[0.940…,0.045…,0.050…,1]` (the same
length, so only that string changes), waited for og dev's reload line and
captured again.

| Reload | og dev total (rebuild + swap) | write to reload line |
|---|---|---|
| 1 | 102.1 ms (46.2 + 55.9) | 145 ms |
| 2 | 112.8 ms (55.9 + 56.9) | 193 ms |
| 3 | 111.6 ms (53.4 + 58.2) | 251 ms |
| 4 | 99.7 ms (52.0 + 47.7) | 164 ms |
| 5 | 99.2 ms (47.4 + 51.8) | 169 ms |
| 6 | 112.6 ms (52.6 + 60.0) | 214 ms |
| 7 | 90.0 ms (45.0 + 45.0) | 171 ms |
| 8 | 99.6 ms (54.0 + 45.6) | 174 ms |

Every reload is within the 300 ms budget on og dev's own figure (worst
112.8 ms) and on the driver's wall clock (worst 251 ms), which adds the
watcher's 20 ms poll and the driver's own polling and start-up. Rebuild
includes the incremental cook (1 of 16 outputs, 22 to 28 ms). The world was
kept (each reload's report: no type dropped or migrated).

**Missed before the fix.** The first run (same commands, 5 edits, 93 to
141 ms) reloaded within budget but the frame never changed: the reload
re-ran the script and kept the models already loaded, so the Model drew the
old meshes. Fixed in og-runtime (3d7ad18: changed cooked models load again
into their asset index after a swapped reload). After the fix the hull is red
after every odd edit and grey after every even one (10,105 red pixels against
0; `m4-gltf-reload.json`), and `packages/engine/tests/reload-asset.test.ts`
guards it.

## M5: a sweep over a tunable yields 8 frames in at most 1 s native

```
og run <project> --port 47914
og probe sweep '{"id":"trickster","parameter":"Spin.speed","values":[-3,-2,-1,0,1,2,3,4],"steps":30}' --port 47914
```

Five sweeps, each 8 distinct 1280×720 frames: 559, 298, 422, 450 and 405 ms
for the whole `og probe sweep` round trip (process start-up, the call, 8 ×
(restore, set, 30 ticks, capture, PNG), the base64 answer). Within 1 s.
