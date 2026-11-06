<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`procgen`](../procgen.md) · [index](../index.md)

<a id="type-NoiseKind"></a>

### `NoiseKind` (enum)

Which gradient noise.

A string: one of the variants below. The zero value is `"perlin"`.

| Variant | Doc |
|---|---|
| `"perlin"` | Ken Perlin's improved noise (2002): 0 on every whole-number point, smooth between. |
| `"simplex"` | Simplex noise (Perlin 2001, Gustavson's construction): fewer directional artefacts, cheaper in 3D. |

```ogt
enum NoiseKind {
  doc "Which gradient noise."
  variant perlin "Ken Perlin's improved noise (2002): 0 on every whole-number point, smooth between."
  variant simplex "Simplex noise (Perlin 2001, Gustavson's construction): fewer directional artefacts, cheaper in 3D."
}
```

<sub>Module `procgen` line 5</sub>
