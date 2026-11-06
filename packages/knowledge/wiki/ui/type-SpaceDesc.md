<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-SpaceDesc"></a>

### `SpaceDesc` (desc struct)

Empty space by the style's scale.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `step` | `SpaceStep` | `"md"` | — | — | — | How much. |
| `grow` | `boolean` | `false` | — | — | — | Take all the space left in its row or column instead (pushes what follows to the end). |

```ogt
struct SpaceDesc {
  doc "Empty space by the style's scale."
  desc true
  field step SpaceStep "How much."
  field grow bool "Take all the space left in its row or column instead (pushes what follows to the end)."
}
```

<sub>`api.ogt` line 19058</sub>
