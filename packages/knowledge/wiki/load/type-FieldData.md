<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`load`](../load.md) · [index](../index.md)

<a id="type-FieldData"></a>

### `FieldData` (struct)

Lossless sampled scalar/vector lattice from type field; x fastest, then y, then z, channels interleaved. No normalization or resampling.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `dimensions` | `[number, number, number]` | `[0, 0, 0]` | — | — | — | Positive sample counts along x, y, z. |
| `channels` | `number` | `0` | — | — | — | One to four scalar channels per cell; units and meaning belong to the author. |
| `origin` | `[number, number, number]` | `[0, 0, 0]` | — | — | — | Coordinates of the first lattice sample. |
| `spacing` | `[number, number, number]` | `[0, 0, 0]` | — | — | — | Positive distance between samples on each axis, in authored coordinate units. |
| `values` | `Float32Array` | — | — | — | — | Finite samples; dimensions product times channels, at most 16,777,216 scalars. |

```ogt
struct FieldData {
  doc "Lossless sampled scalar/vector lattice from type field; x fastest, then y, then z, channels interleaved. No normalization or resampling."
  field dimensions Array.u32.3 "Positive sample counts along x, y, z."
  field channels u32 "One to four scalar channels per cell; units and meaning belong to the author."
  field origin Array.f32.3 "Coordinates of the first lattice sample."
  field spacing Array.f32.3 "Positive distance between samples on each axis, in authored coordinate units."
  field values Float32Array "Finite samples; dimensions product times channels, at most 16,777,216 scalars."
}
```

<sub>Module `load` line 8</sub>
