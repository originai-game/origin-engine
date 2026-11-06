<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`timeline`](../timeline.md) · [index](../index.md)

<a id="type-TimelineInfo"></a>

### `TimelineInfo` (struct)

A loaded timeline.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `asset` | `TimelineAsset` | `0` | — | — | — | Its handle. |
| `name` | `string` | `""` | — | — | — | Its `name`. |
| `duration` | `number` | `0` | — | `s` | — | Its length. |
| `tracks` | `readonly TimelineTrackInfo[]` | `[]` | — | — | — | Its tracks, in file order. |
| `paths` | `readonly string[]` | `[]` | — | — | — | The paths it declares. |
| `markers` | `readonly string[]` | `[]` | — | — | — | Its markers' names, in time order. |

```ogt
struct TimelineInfo {
  doc "A loaded timeline."
  field asset TimelineAsset "Its handle."
  field name String "Its `name`."
  field duration f32 "Its length." {
    unit s
  }
  field tracks List.TimelineTrackInfo "Its tracks, in file order."
  field paths List.String "The paths it declares."
  field markers List.String "Its markers' names, in time order."
}
```

<sub>`api.ogt` line 26191</sub>
