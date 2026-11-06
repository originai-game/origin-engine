<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`stream`](../stream.md) · [index](../index.md)

<a id="record-CellMember"></a>

### `og.CellMember` (component)

Which cell an entity came from: every entity a cell's sub-scene spawns carries one, and when the cell unloads every entity naming it, with its descendants, is despawned, wherever it has moved. Remove it from an entity to keep it past its cell's unload (an item the player picked up); the cell spawns the item again from its sub-scene when it loads again.

Token `og.CellMember`, registered under the schema name `og.CellMember`; its value type is `CellMember`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `cell` | `Entity` | `0` | — | — | read only | The cell's entity. |

```ogt
component CellMember {
  doc "Which cell an entity came from: every entity a cell's sub-scene spawns carries one, and when the cell unloads every entity naming it, with its descendants, is despawned, wherever it has moved. Remove it from an entity to keep it past its cell's unload (an item the player picked up); the cell spawns the item again from its sub-scene when it loads again."
  field cell Entity "The cell's entity." {
    readonly true
  }
}
```

<sub>`api.ogt` line 25412</sub>
