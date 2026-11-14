<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`stream`](../stream.md) · [index](../index.md)

<a id="record-StreamCell"></a>

### `og.StreamCell` (component)

A world cell: loads its sub-scene's entities while an og.StreamAnchor is near its box, and despawns them when none is (the module's doc). Its box (or sphere) is the og.Volume on its entity; without one the cell is a point at the entity's position. The volume's `fade` is not read: `load` and `unload` are the cell's distances.

Also known as: open world streaming, load parts of the world near the player, level streaming, world partition

Token `og.StreamCell`, registered under the schema name `og.StreamCell`; its value type is `StreamCell`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `scene` | `Asset` | `0` | — | — | — | The sub-scene it loads: set from the entity's `scene "<id>"` line when its scene spawns, or og.load.scene's handle for a cell spawned by game code. 0: none (the cell never loads). |
| `load` | `number` | `0` | ≥ 0 | `m` | — | Loads while an anchor is within this distance of the box (0: only while one is inside it). |
| `unload` | `number` | `0` | ≥ 0 | `m` | — | Unloads once every anchor is farther than this from the box, so an anchor on the edge does not load and unload it every tick; 0: 1.25 × load. Less than `load` is taken as `load`. |
| `pin` | `boolean` | `false` | — | — | — | Loaded whatever the anchors while true (a cell game code needs, such as a quest's destination); set it false and the anchors decide again. |
| `persist` | `boolean` | `false` | — | — | — | Keep what happened in it: on unload its entities as they are are kept in the world's og.cells state (in snapshots, saves and the hash) and come back so when it loads again. False: it loads as authored every time. See `world §5`. |
| `budget` | `number` | `0` | — | — | — | The most of its entities spawned per tick while it loads (0: all in one tick). `loaded` turns true, and CellEvent is sent, in the tick its last entity spawns; a cell unwanted before then despawns what it spawned and sends nothing. See `world §5`. |
| `loaded` | `boolean` | `false` | — | — | read only | Written by the engine: true while the sub-scene's entities are in the world (from the tick it loaded until the tick it unloaded). |

```ogt
component StreamCell {
  aliases "open world streaming, load parts of the world near the player, level streaming, world partition"
  doc "A world cell: loads its sub-scene's entities while an og.StreamAnchor is near its box, and despawns them when none is (the module's doc). Its box (or sphere) is the og.Volume on its entity; without one the cell is a point at the entity's position. The volume's `fade` is not read: `load` and `unload` are the cell's distances."
  field scene Asset "The sub-scene it loads: set from the entity's `scene \"<id>\"` line when its scene spawns, or og.load.scene's handle for a cell spawned by game code. 0: none (the cell never loads)."
  field load f32 "Loads while an anchor is within this distance of the box (0: only while one is inside it)." {
    unit m
    min 0
  }
  field unload f32 "Unloads once every anchor is farther than this from the box, so an anchor on the edge does not load and unload it every tick; 0: 1.25 × load. Less than `load` is taken as `load`." {
    unit m
    min 0
  }
  field pin bool "Loaded whatever the anchors while true (a cell game code needs, such as a quest's destination); set it false and the anchors decide again."
  field persist bool "Keep what happened in it: on unload its entities as they are are kept in the world's og.cells state (in snapshots, saves and the hash) and come back so when it loads again. False: it loads as authored every time." {
    guide "world §5"
  }
  field budget u32 "The most of its entities spawned per tick while it loads (0: all in one tick). `loaded` turns true, and CellEvent is sent, in the tick its last entity spawns; a cell unwanted before then despawns what it spawned and sends nothing." {
    guide "world §5"
  }
  field loaded bool "Written by the engine: true while the sub-scene's entities are in the world (from the tick it loaded until the tick it unloaded)." {
    readonly true
  }
}
```

<sub>Module `stream` line 6</sub>
