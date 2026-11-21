<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`flow`](../flow.md) · [index](../index.md)

<a id="type-FlowHotspot"></a>

### `FlowHotspot` (struct)

A place on the picture that takes a choice (`#hotspot:<id>` on the choice, a `track hotspot` key in its segment), while the window is open.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `choice` | `number` | `0` | — | — | — | The ChoiceView.index it takes. |
| `id` | `string` | `""` | — | — | — | The hotspot's id. |
| `x` | `number` | `0` | — | `ratio` | — | Its left edge, a fraction of the video frame from its left. |
| `y` | `number` | `0` | — | `ratio` | — | Its top edge, from the frame's top. |
| `width` | `number` | `0` | — | `ratio` | — | Its width, a fraction of the frame's. |
| `height` | `number` | `0` | — | `ratio` | — | Its height. |

```ogt
struct FlowHotspot {
  doc "A place on the picture that takes a choice (`#hotspot:<id>` on the choice, a `track hotspot` key in its segment), while the window is open."
  field choice u32 "The ChoiceView.index it takes."
  field id String "The hotspot's id."
  field x f32 "Its left edge, a fraction of the video frame from its left." {
    unit ratio
  }
  field y f32 "Its top edge, from the frame's top." {
    unit ratio
  }
  field width f32 "Its width, a fraction of the frame's." {
    unit ratio
  }
  field height f32 "Its height." {
    unit ratio
  }
}
```

<sub>Module `flow` line 89</sub>
