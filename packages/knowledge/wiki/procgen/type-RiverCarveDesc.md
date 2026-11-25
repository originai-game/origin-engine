<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`procgen`](../procgen.md) · [index](../index.md)

<a id="type-RiverCarveDesc"></a>

### `RiverCarveDesc` (desc struct)

Drainage-accumulation river carving pass.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `threshold` | `number` | `400` | ≥ 1 | — | — | Upstream sample count required before carving a channel. |
| `depth` | `number` | `2` | ≥ 0 | `m` | — | Maximum carved channel depth. |
| `width` | `number` | `6` | ≥ 0.1 | `m` | — | Half-width of carved river channels. |

```ogt
struct RiverCarveDesc {
  doc "Drainage-accumulation river carving pass."
  desc true
  field threshold u32 "Upstream sample count required before carving a channel." {
    default 400
    min 1
  }
  field depth f32 "Maximum carved channel depth." {
    default 2
    min 0
    unit m
  }
  field width f32 "Half-width of carved river channels." {
    default 6
    min 0.1
    unit m
  }
}
```

<sub>Module `procgen` line 367</sub>
