<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`camera`](../camera.md) · [index](../index.md)

<a id="record-CameraCut"></a>

### `og.CameraCut` (event)

The live virtual camera changed (sent by the og.camera system; read the tick after).

Token `og.CameraCut`, registered under the schema name `og.CameraCut`; its value type is `CameraCut`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `from` | `Entity` | `0` | — | — | — | The camera that was live (0: none). |
| `to` | `Entity` | `0` | — | — | — | The camera now live (0: none). |
| `blend` | `number` | `0` | — | `s` | — | The blend's length in seconds (0: a cut). |

```ogt
event CameraCut {
  doc "The live virtual camera changed (sent by the og.camera system; read the tick after)."
  field from Entity "The camera that was live (0: none)."
  field to Entity "The camera now live (0: none)."
  field blend f32 "The blend's length in seconds (0: a cut)." {
    unit s
  }
}
```

<sub>`api.ogt` line 3759</sub>
