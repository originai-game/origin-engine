<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-UiOverlap"></a>

### `UiOverlap` (struct)

Two elements overlapping where neither declared it (docs/ui.md §2).

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `path` | `string` | `""` | — | — | — | The element placed over the other (absolute, anchored or floating), or the later of two siblings in the flow. |
| `other` | `string` | `""` | — | — | — | The other element. |
| `rect` | `UiRect` | `{ x: 0, y: 0, width: 0, height: 0 }` | — | — | — | Where they overlap. |
| `inFlow` | `boolean` | `false` | — | — | — | Both are in their parent's flow: a layout bug, not a placement. |

```ogt
struct UiOverlap {
  doc "Two elements overlapping where neither declared it (docs/ui.md §2)."
  field path String "The element placed over the other (absolute, anchored or floating), or the later of two siblings in the flow."
  field other String "The other element."
  field rect UiRect "Where they overlap."
  field inFlow bool "Both are in their parent's flow: a layout bug, not a placement."
}
```

<sub>Module `ui` line 5166</sub>
