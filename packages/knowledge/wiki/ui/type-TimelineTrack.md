<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-TimelineTrack"></a>

### `TimelineTrack` (struct)

Keys over one prop of the element the timeline plays on, or of a descendant by its path below it (`Stars/Star#2`). Before its first key it holds that key's value, after its last the last.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `target` | `string` | `""` | — | — | — | A path below the play's element (empty: the element itself). |
| `prop` | `UiProp` | `"x"` | — | — | — | The prop. |
| `keys` | `readonly TimelineKey[]` | `[]` | — | — | — | In time order. |

```ogt
struct TimelineTrack {
  doc "Keys over one prop of the element the timeline plays on, or of a descendant by its path below it (`Stars/Star#2`). Before its first key it holds that key's value, after its last the last."
  field target String "A path below the play's element (empty: the element itself)."
  field prop UiProp "The prop."
  field keys List.TimelineKey "In time order."
}
```

<sub>`api.ogt` line 20721</sub>
