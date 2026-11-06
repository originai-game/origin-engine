<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`motion`](../motion.md) · [index](../index.md)

<a id="type-MotionCapabilities"></a>

### `MotionCapabilities` (struct)

Where the motion model runs and what it knows. Presentation: a frame's answer, not the tick's; never branch the simulation on it.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `backend` | `MotionBackend` | `"none"` | — | — | — | gpu once the weights are resident; none before, or when they failed. |
| `reason` | `string` | `""` | — | — | — | Why the backend is none: loading, the reason a load failed, or no GPU adapter (empty for gpu). |
| `modelBytes` | `number` | `0` | — | `bytes` | — | The size of cooked/motion/model.ogb, once read (0 before). |

```ogt
struct MotionCapabilities {
  doc "Where the motion model runs and what it knows. Presentation: a frame's answer, not the tick's; never branch the simulation on it."
  field backend MotionBackend "gpu once the weights are resident; none before, or when they failed."
  field reason String "Why the backend is none: loading, the reason a load failed, or no GPU adapter (empty for gpu)."
  field modelBytes f64 "The size of cooked/motion/model.ogb, once read (0 before)." {
    unit bytes
  }
}
```

<sub>`api.ogt` line 26999</sub>
