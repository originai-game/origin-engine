<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`stream`](../stream.md) · [index](../index.md)

<a id="record-StreamAnchor"></a>

### `og.StreamAnchor` (component)

What loads cells: every entity with one (the player, a vehicle, each player of a networked game) loads the cells within their StreamCell.load of it, from its world position. Without any anchor the og.Camera resource's position anchors (refused in an og.net session).

Token `og.StreamAnchor`, registered under the schema name `og.StreamAnchor`; its value type is `StreamAnchor`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `reach` | `number` | `0` | ≥ 0 | `m` | — | Added to every cell's load and unload distances for this anchor: a fast vehicle reaches further than a walker. |
| `maxSpeed` | `number` | `0` | ≥ 0 | `mps` | — | The fastest it moves, for reading ahead: a cell's asset group (`cell` in its sub-scene's stream block) is fetched once this anchor is within the cell's load distance plus reach plus 2 seconds at this speed, so it has landed when the cell loads and the tick never waits for it. Read only for fetching, never by a tick (0: fetched at the load distance). |

```ogt
component StreamAnchor {
  doc "What loads cells: every entity with one (the player, a vehicle, each player of a networked game) loads the cells within their StreamCell.load of it, from its world position. Without any anchor the og.Camera resource's position anchors (refused in an og.net session)."
  field reach f32 "Added to every cell's load and unload distances for this anchor: a fast vehicle reaches further than a walker." {
    unit m
    min 0
  }
  field maxSpeed f32 "The fastest it moves, for reading ahead: a cell's asset group (`cell` in its sub-scene's stream block) is fetched once this anchor is within the cell's load distance plus reach plus 2 seconds at this speed, so it has landed when the cell loads and the tick never waits for it. Read only for fetching, never by a tick (0: fetched at the load distance)." {
    unit mps
    min 0
  }
}
```

<sub>Module `stream` line 33</sub>
