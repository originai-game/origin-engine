<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-LootLabelResult"></a>

### `LootLabelResult` (struct)

What a loot label answered.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `clicked` | `boolean` | `false` | — | — | — | Pressed this tick (pick it up). |
| `hovered` | `boolean` | `false` | — | — | — | Under the pointer (show its tooltip). |
| `info` | `AnchorInfo` | `{ inView: false, onScreen: false, clamped: false, lod: 0, distance: 0, x: 0, y: 0, angle: 0, scale: 0, alpha: 0 }` | — | — | — | Where it stands. |

```ogt
struct LootLabelResult {
  doc "What a loot label answered."
  field clicked bool "Pressed this tick (pick it up)."
  field hovered bool "Under the pointer (show its tooltip)."
  field info AnchorInfo "Where it stands."
}
```

<sub>Module `ui` line 6837</sub>
