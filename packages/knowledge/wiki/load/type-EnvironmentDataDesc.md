<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`load`](../load.md) · [index](../index.md)

<a id="type-EnvironmentDataDesc"></a>

### `EnvironmentDataDesc` (desc struct)

How og.load.environmentData reads an HDR environment.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `size` | `number` | `128` | — | — | — | The prefiltered cube's face size at its sharpest level: a power of two from 16 to 2048 (and at most LoadBudget.textureSize), as a cooked environment's `size`. |
| `intensity` | `number` | `1` | ≥ 0 | — | — | Multiplies the image, as a cooked environment's `intensity`. |
| `budget` | `LoadBudget` | `{}` | — | — | — | The limits it is held to: bytes, and textureSize for the image's width and height. |

```ogt
struct EnvironmentDataDesc {
  doc "How og.load.environmentData reads an HDR environment."
  desc true
  field size u32 "The prefiltered cube's face size at its sharpest level: a power of two from 16 to 2048 (and at most LoadBudget.textureSize), as a cooked environment's `size`." {
    default 128
  }
  field intensity f32 "Multiplies the image, as a cooked environment's `intensity`." {
    default 1
    min 0
  }
  field budget LoadBudget "The limits it is held to: bytes, and textureSize for the image's width and height."
}
```

<sub>Module `load` line 529</sub>
