<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`probe`](../probe.md) · [index](../index.md)

<a id="type-ViewFrames"></a>

### `ViewFrames` (struct)

The frames of one tick from several viewpoints.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `tick` | `number` | `0` | — | — | — | The tick every frame shows. |
| `frames` | `readonly ViewFrame[]` | `[]` | — | — | — | One per viewpoint, in order. |

```ogt
struct ViewFrames {
  doc "The frames of one tick from several viewpoints."
  field tick u32 "The tick every frame shows."
  field frames List.ViewFrame "One per viewpoint, in order."
}
```

<sub>Module `probe` line 218</sub>
