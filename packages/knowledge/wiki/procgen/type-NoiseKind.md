<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`procgen`](../procgen.md) · [index](../index.md)

<a id="type-NoiseKind"></a>

### `NoiseKind` (enum)

Which procedural noise basis or fractal shaping.

A string: one of the variants below. The zero value is `"perlin"`.

| Variant | Doc |
|---|---|
| `"perlin"` | C2 quintic gradient noise (Perlin 2002): 0 on lattice points, smooth between. |
| `"simplex"` | Simplex noise (Perlin 2001, Gustavson's construction): fewer directional artefacts, cheaper in 3D. |
| `"worley"` | Cellular Worley F1 distance noise, remapped to [-1, 1]. |
| `"worleyEdge"` | Cellular Worley F2 - F1 edge/ridge distance, remapped to [-1, 1]. |
| `"ridged"` | Ridged multifractal gradient noise (sharp mountain ridges and cracks). |
| `"billow"` | Billowy turbulence gradient noise (puffy cumulus/rolling hills). |

```ogt
enum NoiseKind {
  doc "Which procedural noise basis or fractal shaping."
  variant perlin "C2 quintic gradient noise (Perlin 2002): 0 on lattice points, smooth between."
  variant simplex "Simplex noise (Perlin 2001, Gustavson's construction): fewer directional artefacts, cheaper in 3D."
  variant worley "Cellular Worley F1 distance noise, remapped to [-1, 1]."
  variant worleyEdge "Cellular Worley F2 - F1 edge/ridge distance, remapped to [-1, 1]."
  variant ridged "Ridged multifractal gradient noise (sharp mountain ridges and cracks)."
  variant billow "Billowy turbulence gradient noise (puffy cumulus/rolling hills)."
}
```

<sub>Module `procgen` line 7</sub>
