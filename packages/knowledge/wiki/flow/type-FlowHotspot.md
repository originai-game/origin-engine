<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`flow`](../flow.md) · [index](../index.md)

<a id="type-FlowHotspot"></a>

### `FlowHotspot` (struct)

A place on the picture: either a choice (`#hotspot:<id>` on the choice, a `track hotspot` key in its segment) while the window is open, or a non-cutting `inspect` hotspot (triggered with og.flow.inspect).

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `choice` | `number \| null` | `null` | — | — | — | The ChoiceView.index it takes; null for an `inspect` hotspot, which does not cut. |
| `id` | `string` | `""` | — | — | — | The hotspot's id. |
| `kind` | `string` | `""` | — | — | — | `choice` (takes a choice) or `inspect` (non-cutting investigation point, fires og.FlowInspected). |
| `shape` | `string` | `""` | — | — | — | `rect` or `circle`. |
| `x` | `number` | `0` | — | `ratio` | — | Its left edge, a fraction of the video frame from its left. |
| `y` | `number` | `0` | — | `ratio` | — | Its top edge, from the frame's top. |
| `width` | `number` | `0` | — | `ratio` | — | Its width, a fraction of the frame's. |
| `height` | `number` | `0` | — | `ratio` | — | Its height. |
| `label` | `string` | `""` | — | — | — | Its localized label (from the hotspot key's `label` i18n key, or the choice's text). |
| `inspected` | `boolean` | `false` | — | — | — | Whether this `inspect` hotspot has already been triggered in this segment. |

```ogt
struct FlowHotspot {
  doc "A place on the picture: either a choice (`#hotspot:<id>` on the choice, a `track hotspot` key in its segment) while the window is open, or a non-cutting `inspect` hotspot (triggered with og.flow.inspect)."
  field choice Option.u32 "The ChoiceView.index it takes; null for an `inspect` hotspot, which does not cut."
  field id String "The hotspot's id."
  field kind String "`choice` (takes a choice) or `inspect` (non-cutting investigation point, fires og.FlowInspected)."
  field shape String "`rect` or `circle`."
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
  field label String "Its localized label (from the hotspot key's `label` i18n key, or the choice's text)."
  field inspected bool "Whether this `inspect` hotspot has already been triggered in this segment."
}
```

<sub>Module `flow` line 105</sub>
