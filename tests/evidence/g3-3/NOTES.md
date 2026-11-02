# G3-3: world cells (batch 1: sub-scenes, cells, events, order)

`cells/` is one 3D world: a road and a rover with an `og.StreamAnchor`,
plus five cells 32 m apart along the road (`scenes/world.ogt`). Each cell
is an entity with `scene "scenes/cells/<name>.ogt"` and
`StreamCell { size 24 12 24 load 12 unload 15 }`. Its sub-scene is one of
three unnamed scenes, so the village and the forest are placed twice:

- the village: a slab with a static box collider, two houses with static
  colliders, and six crates stacked 2 to 5 m up as dynamic bodies;
- the forest: the slab, seven trees with cylinder colliders, and four
  balls dropped from 3 m;
- the quarry: the slab and a pyramid of ten crates 30 cm above it.

The rover drives at 12 m/s from x = -36 (`src/main.ts`, `Drive`), and the
camera follows it. Cell *i*'s box spans x = 32*i* ± 12. The cell loads the
tick the rover is within 12 m of the box (x ≥ 32*i* − 24) and unloads once
the rover is 15 m past it (x > 32*i* + 27). The caption lists each cell's
last `og.CellEvent` with its entity count and tick, read by the `Report`
system; its first line is the tick the systems ran, one before the
screenshot's.

## Run on the Mac satellite (`metal`, macos-27, a dev build of this branch)

```sh
og run tests/evidence/g3-3/cells --headless --size 1280x720 --out <dir> --json --commands \
  "0.5:screenshot:empty.png;1.1:screenshot:cell0-lands.png;2.5:screenshot:cell0-settled.png;4.0:screenshot:cell1-lands.png;5.6:screenshot:cell0-gone.png;7.0:screenshot:quarry.png;9.0:screenshot:quarry-settled.png;9.5:dump_stream;9.6:quit"
```

Two runs ended on the same hash, `077f8931ff753838` at tick 576.

## What each frame shows

- `empty.png` (tick 30, rover at x −30): the road and the rover, with no
  cell loaded.
- `cell0-lands.png` (tick 66): cell-0 loaded at tick 61 (x −23.8, the
  first tick past −24) with 9 entities. The houses stand on the slab and
  the crates are still in the air.
- `cell0-settled.png` (tick 150): the crates have fallen onto the slab's
  collider and stand there stacked, beside the road.
- `cell1-lands.png` (tick 240): cell-1, the forest, loaded at tick 221
  (x 8.2) with 12 entities, and its balls bounce on its slab. cell-0 is
  still loaded behind the rover.
- `cell0-gone.png` (tick 336): cell-0 unloaded at tick 316 (x 27.2, the
  first tick past 27). Its 9 entities are gone, and the road runs over the
  bare ground behind the rover.
- `quarry.png` (tick 420) and `quarry-settled.png` (tick 540): cell-2
  loaded at tick 381 (x 40.2) with 11 entities, and its pyramid settles on
  the slab. cell-1 unloaded at tick 476 (x 59.2).

`dump_stream-000570.json`'s `cells` agree with the frames: cell-0 loaded
at 61 and unloaded at 316, cell-1 loaded at 221 and unloaded at 476,
cell-2 loaded at 381 with 11 entities, cell-3 (the second village) loaded
at 541 with 9, and cell-4 never loaded.

## Tests

- `crates/og-runtime/tests/cells.rs`: the load and unload distances, the
  placement at the cell's pose, members and children despawned, a replay
  and a restored save going tick for tick across a load and an unload (the
  well's entity ids and the last hash pinned), the camera refused in a
  session, and a plain sub-scene spawned under its entity.
- `crates/og-cook/tests/subscene.rs`: the `subscenes` table, the cycle
  refused naming the chain, and a `scene` line that names no scene.
- `prefabs.rs`'s `BY_HAND_HASH` is unmoved: a world without cells hashes
  as before.
