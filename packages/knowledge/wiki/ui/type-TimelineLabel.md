<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-TimelineLabel"></a>

### `TimelineLabel` (struct)

A named time, for seeking (PlayDesc.fromLabel).

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `name` | `string` | `""` | — | — | — | Its name. |
| `atMs` | `number` | `0` | — | `ms` | — | Its time. |

```ogt
struct TimelineLabel {
  doc "A named time, for seeking (PlayDesc.fromLabel)."
  field name String "Its name."
  field atMs f32 "Its time." {
    unit ms
  }
}
```

<sub>Module `ui` line 1012</sub>
