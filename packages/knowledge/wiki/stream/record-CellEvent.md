<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`stream`](../stream.md) · [index](../index.md)

<a id="record-CellEvent"></a>

### `og.CellEvent` (event)

A cell loaded or unloaded: sent in the tick it happened, before any system runs, so the tick's systems read it with the cell's entities already spawned (or despawned).

Token `og.CellEvent`, registered under the schema name `og.CellEvent`; its value type is `CellEvent`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `cell` | `Entity` | `0` | — | — | — | The cell's entity. |
| `loaded` | `boolean` | `false` | — | — | — | True: its sub-scene's entities spawned; false: its members and their descendants were despawned. |
| `entities` | `number` | `0` | — | — | — | How many entities spawned or were despawned. |

```ogt
event CellEvent {
  doc "A cell loaded or unloaded: sent in the tick it happened, before any system runs, so the tick's systems read it with the cell's entities already spawned (or despawned)."
  field cell Entity "The cell's entity."
  field loaded bool "True: its sub-scene's entities spawned; false: its members and their descendants were despawned."
  field entities u32 "How many entities spawned or were despawned."
}
```

<sub>Module `stream` line 52</sub>
