<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`procgen`](../procgen.md) · [index](../index.md)

<a id="type-NoiseGrad"></a>

### `NoiseGrad` (struct)

Noise value together with its analytical partial derivatives `(dx, dy, dz)` (dz is 0 in 2D).

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `value` | `number` | `0` | — | — | — | Noise sample at the point, about -1 to 1. |
| `dx` | `number` | `0` | — | — | — | Partial derivative along x. |
| `dy` | `number` | `0` | — | — | — | Partial derivative along y. |
| `dz` | `number` | `0` | — | — | — | Partial derivative along z (0 for 2D noise). |

```ogt
struct NoiseGrad {
  doc "Noise value together with its analytical partial derivatives `(dx, dy, dz)` (dz is 0 in 2D)."
  field value f32 "Noise sample at the point, about -1 to 1."
  field dx f32 "Partial derivative along x."
  field dy f32 "Partial derivative along y."
  field dz f32 "Partial derivative along z (0 for 2D noise)."
}
```

<sub>Module `procgen` line 78</sub>
