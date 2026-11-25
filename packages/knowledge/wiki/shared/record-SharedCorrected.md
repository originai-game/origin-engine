<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`shared`](../shared.md) · [index](../index.md)

<a id="record-SharedCorrected"></a>

### `og.SharedCorrected` (event)

The server refused a move of this avatar (faster than its kind allows, or outside the world's bounds) and the engine put it back: at the state the server holds, plus what it moved here since. A game may snap its camera or tell the player.

See `shared-worlds §4`.

Token `og.SharedCorrected`, registered under the schema name `og.SharedCorrected`; its value type is `SharedCorrected`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `sequence` | `number` | `0` | — | — | — | The proposal the correction answers. |

```ogt
event SharedCorrected {
  doc "The server refused a move of this avatar (faster than its kind allows, or outside the world's bounds) and the engine put it back: at the state the server holds, plus what it moved here since. A game may snap its camera or tell the player."
  guide "shared-worlds §4"
  field sequence u32 "The proposal the correction answers."
}
```

<sub>Module `shared` line 116</sub>
