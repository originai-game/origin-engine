<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`sight`](../sight.md) · [index](../index.md)

<a id="record-SightBlocker"></a>

### `og.SightBlocker` (component)

Stops sight on the og.SightMap: the cells whose centre lies inside the og.Volume on its entity (a box or a sphere, at its entity's pose, cut at its own height) hide what is behind them. Without an og.Volume it blocks nothing. A wall thinner than a cell may fall between cell centres: make it at least a cell thick.

See `world §11`.

Also known as: blocks vision, wall blocks line of sight, opaque to sight, vision blocker, sight occluder

Token `og.SightBlocker`, registered under the schema name `og.SightBlocker`; its value type is `SightBlocker`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `enabled` | `boolean` | `true` | — | — | — | Whether it blocks (a door: false while open). |

```ogt
component SightBlocker {
  aliases "blocks vision, wall blocks line of sight, opaque to sight, vision blocker, sight occluder"
  doc "Stops sight on the og.SightMap: the cells whose centre lies inside the og.Volume on its entity (a box or a sphere, at its entity's pose, cut at its own height) hide what is behind them. Without an og.Volume it blocks nothing. A wall thinner than a cell may fall between cell centres: make it at least a cell thick."
  guide "world §11"
  field enabled bool "Whether it blocks (a door: false while open)." {
    default true
  }
}
```

<sub>Module `sight` line 60</sub>
