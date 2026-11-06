<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-ListDesc"></a>

### `ListDesc` (desc struct)

A virtualised list: only the rows in view (and one more each side) are built, fillers keep the scroll range of every row.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `count` | `number` | `0` | — | — | — | How many rows there are. |
| `rowHeight` | `number` | `0` | ≥ 0 | `cssPx` | — | Every row's height; 0 is the density's row height. |
| `gap` | `number` | `0` | — | `cssPx` | — | Space between rows. |
| `width` | `Size \| null` | `null` | — | — | — | Its width; null fits. |
| `height` | `Size \| null` | `null` | — | — | — | Its height (it scrolls inside); null grows. |
| `padding` | `Insets` | `{ left: 0, top: 0, right: 0, bottom: 0 }` | — | — | — | Space inside. |
| `selected` | `number \| null` | `null` | — | — | — | The selected row (controlStrong), for og.ui.list. |

```ogt
struct ListDesc {
  doc "A virtualised list: only the rows in view (and one more each side) are built, fillers keep the scroll range of every row."
  desc true
  field count u32 "How many rows there are."
  field rowHeight f32 "Every row's height; 0 is the density's row height." {
    unit cssPx
    min 0
  }
  field gap f32 "Space between rows." {
    unit cssPx
  }
  field width Option.Size "Its width; null fits."
  field height Option.Size "Its height (it scrolls inside); null grows."
  field padding Insets "Space inside."
  field selected Option.u32 "The selected row (controlStrong), for og.ui.list."
}
```

<sub>`api.ogt` line 16629</sub>
