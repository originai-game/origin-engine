<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-TreeDesc"></a>

### `TreeDesc` (desc struct)

A tree of expandable items; the visible rows are virtualised.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `items` | `readonly TreeItem[]` | `[]` | — | — | — | The items, each after its parent or not: order among siblings is kept. |
| `expanded` | `readonly string[]` | `[]` | — | — | — | The open items' ids (keep TreeResult.expanded). |
| `selected` | `string` | `""` | — | — | — | The selected item's id. |
| `toggleOnPress` | `boolean` | `true` | — | — | — | A press on an item with children opens or closes it as well as choosing it. |
| `rowHeight` | `number \| null` | `null` | — | `cssPx` | — | Every row's height; null is 36 (44 on a phone). |
| `indent` | `number \| null` | `null` | — | `cssPx` | — | Each level's indent; null is 20. |
| `width` | `Size \| null` | `null` | — | — | — | Its width; null grows. |
| `height` | `Size \| null` | `null` | — | — | — | Its height (the rows scroll inside); null fits. |
| `label` | `string` | `""` | — | — | — | What a screen reader calls it. |

```ogt
struct TreeDesc {
  doc "A tree of expandable items; the visible rows are virtualised."
  desc true
  field items List.TreeItem "The items, each after its parent or not: order among siblings is kept."
  field expanded List.String "The open items' ids (keep TreeResult.expanded)."
  field selected String "The selected item's id."
  field toggleOnPress bool "A press on an item with children opens or closes it as well as choosing it." {
    default true
  }
  field rowHeight Option.f32 "Every row's height; null is 36 (44 on a phone)." {
    unit cssPx
  }
  field indent Option.f32 "Each level's indent; null is 20." {
    unit cssPx
  }
  field width Option.Size "Its width; null grows."
  field height Option.Size "Its height (the rows scroll inside); null fits."
  field label String "What a screen reader calls it."
}
```

<sub>Module `ui` line 2636</sub>
