<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`camera`](../camera.md) · [index](../index.md)

<a id="record-CameraCocDropped"></a>

### `og.CameraCocDropped` (event)

A camera CoC blend exceeded MAX_CAMERA_COC_SOURCES after identical-source coalescing. The lowest-weight primitive was dropped and survivors renormalised, producing a defined step rather than fitting a curve. CameraBrain also retains the latest drop and total count for probe and dump.

Token `og.CameraCocDropped`, registered under the schema name `og.CameraCocDropped`; its value type is `CameraCocDropped`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `source` | `readonly number[]` | `[]` | — | — | a list of at most 9 `f32` (`List.f32.9`): reads a frozen copy, writes a whole array | Dropped primitive: focal length mm (0: artistic), fitted gate height / zoom mm, aperture, focus distance m, focus range m, falloff, radius cap / output height, tilt half-height, pre-renormalisation weight. |
| `frame` | `number` | `0` | — | — | — | The fixed simulation frame (tick) when the camera evaluated the drop, not a wall-clock or display-frame counter. |
| `count` | `number` | `0` | — | — | — | CameraBrain's cumulative drop count, including this event. |

```ogt
event CameraCocDropped {
  doc "A camera CoC blend exceeded MAX_CAMERA_COC_SOURCES after identical-source coalescing. The lowest-weight primitive was dropped and survivors renormalised, producing a defined step rather than fitting a curve. CameraBrain also retains the latest drop and total count for probe and dump."
  field source List.f32.9 "Dropped primitive: focal length mm (0: artistic), fitted gate height / zoom mm, aperture, focus distance m, focus range m, falloff, radius cap / output height, tilt half-height, pre-renormalisation weight."
  field frame u32 "The fixed simulation frame (tick) when the camera evaluated the drop, not a wall-clock or display-frame counter."
  field count u32 "CameraBrain's cumulative drop count, including this event."
}
```

<sub>Module `camera` line 249</sub>
