<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`probe`](../probe.md) · [index](../index.md)

<a id="type-SweepDesc"></a>

### `SweepDesc` (desc struct)

A parameter sweep: for each value, restore the snapshot taken at the call, patch, advance and capture.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `id` | `string` | `""` | — | — | — | The probe id of the entity. |
| `parameter` | `string` | `""` | — | — | — | The parameter, as `Health.hp`. |
| `values` | `readonly unknown[]` | `[]` | — | — | — | The values to try, in order. |
| `steps` | `number` | `0` | — | — | — | Fixed steps to advance after each patch. |
| `width` | `number` | `0` | — | — | — | Capture width; 0 is the render target's. |
| `height` | `number` | `0` | — | — | — | Capture height; 0 is the render target's. |

```ogt
struct SweepDesc {
  doc "A parameter sweep: for each value, restore the snapshot taken at the call, patch, advance and capture."
  desc true
  field id String "The probe id of the entity."
  field parameter String "The parameter, as `Health.hp`."
  field values List.Json "The values to try, in order."
  field steps u32 "Fixed steps to advance after each patch."
  field width u32 "Capture width; 0 is the render target's."
  field height u32 "Capture height; 0 is the render target's."
}
```

<sub>`api.ogt` line 24163</sub>
