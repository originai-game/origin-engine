<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-UiLayoutStats"></a>

### `UiLayoutStats` (struct)

What the last frame rebuilt and laid out (docs/ui.md §2).

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `elements` | `number` | `0` | — | — | — | Elements built. |
| `rebuilt` | `number` | `0` | — | — | — | Elements whose inputs changed since the tick before (or are new). |
| `laidOut` | `number` | `0` | — | — | — | Elements laid out; the rest kept their last layout. |
| `keptCommands` | `number` | `0` | — | — | — | Elements whose draw commands were kept from the tick before. |

```ogt
struct UiLayoutStats {
  doc "What the last frame rebuilt and laid out (docs/ui.md §2)."
  field elements u32 "Elements built."
  field rebuilt u32 "Elements whose inputs changed since the tick before (or are new)."
  field laidOut u32 "Elements laid out; the rest kept their last layout."
  field keptCommands u32 "Elements whose draw commands were kept from the tick before."
}
```

<sub>Module `ui` line 2587</sub>
