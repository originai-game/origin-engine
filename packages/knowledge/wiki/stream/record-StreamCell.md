<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`stream`](../stream.md) · [index](../index.md)

<a id="record-StreamCell"></a>

### `og.StreamCell` (component)

A world cell: loads its sub-scene's entities while an og.StreamAnchor is near its box, and despawns them when none is (the module's doc). The box is the entity's world position ± half `size`, turned with it; a zero size is a point.

Token `og.StreamCell`, registered under the schema name `og.StreamCell`; its value type is `StreamCell`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `scene` | `Asset` | `0` | — | — | — | The sub-scene it loads: set from the entity's `scene "<id>"` line when its scene spawns, or og.load.scene's handle for a cell spawned by game code. 0: none (the cell never loads). |
| `size` | `Vector3` | `{ x: 0, y: 0, z: 0 }` | — | `m` | — | The box's full extents, in metres along the entity's axes (its scale does not apply). Zero: a point at the entity's position. |
| `load` | `number` | `0` | ≥ 0 | `m` | — | Loads while an anchor is within this distance of the box (0: only while one is inside it). |
| `unload` | `number` | `0` | ≥ 0 | `m` | — | Unloads once every anchor is farther than this from the box, so an anchor on the edge does not load and unload it every tick; 0: 1.25 × load. Less than `load` is taken as `load`. |
| `pin` | `boolean` | `false` | — | — | — | Loaded whatever the anchors while true (a cell game code needs, such as a quest's destination); set it false and the anchors decide again. |
| `persist` | `boolean` | `false` | — | — | — | Keep what happened in it: when it unloads, its entities as they are (moved, damaged, opened; every component and name) are kept in the world's og.cells state, carried by snapshots, saves and the hash, and they come back as they were when it loads again instead of its sub-scene as authored. False: it loads as authored every time. |
| `budget` | `number` | `0` | — | — | — | The most of its entities spawned per tick while it loads, so a large cell spreads over consecutive ticks instead of one long one (0: all in one tick). `loaded` turns true, and CellEvent is sent, in the tick its last entity spawns; a cell that stops being wanted before then despawns what it spawned and sends nothing. |
| `loaded` | `boolean` | `false` | — | — | read only | Written by the engine: true while the sub-scene's entities are in the world (from the tick it loaded until the tick it unloaded). |

```ogt
component StreamCell {
  doc "A world cell: loads its sub-scene's entities while an og.StreamAnchor is near its box, and despawns them when none is (the module's doc). The box is the entity's world position ± half `size`, turned with it; a zero size is a point."
  field scene Asset "The sub-scene it loads: set from the entity's `scene \"<id>\"` line when its scene spawns, or og.load.scene's handle for a cell spawned by game code. 0: none (the cell never loads)."
  field size Vector3 "The box's full extents, in metres along the entity's axes (its scale does not apply). Zero: a point at the entity's position." {
    unit m
  }
  field load f32 "Loads while an anchor is within this distance of the box (0: only while one is inside it)." {
    unit m
    min 0
  }
  field unload f32 "Unloads once every anchor is farther than this from the box, so an anchor on the edge does not load and unload it every tick; 0: 1.25 × load. Less than `load` is taken as `load`." {
    unit m
    min 0
  }
  field pin bool "Loaded whatever the anchors while true (a cell game code needs, such as a quest's destination); set it false and the anchors decide again."
  field persist bool "Keep what happened in it: when it unloads, its entities as they are (moved, damaged, opened; every component and name) are kept in the world's og.cells state, carried by snapshots, saves and the hash, and they come back as they were when it loads again instead of its sub-scene as authored. False: it loads as authored every time."
  field budget u32 "The most of its entities spawned per tick while it loads, so a large cell spreads over consecutive ticks instead of one long one (0: all in one tick). `loaded` turns true, and CellEvent is sent, in the tick its last entity spawns; a cell that stops being wanted before then despawns what it spawned and sends nothing."
  field loaded bool "Written by the engine: true while the sub-scene's entities are in the world (from the tick it loaded until the tick it unloaded)." {
    readonly true
  }
}
```

<sub>`api.ogt` line 25378</sub>
