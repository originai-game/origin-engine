<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`procgen`](../procgen.md) · [index](../index.md)

<a id="type-FractureResult"></a>

### `FractureResult` (struct)

Solid pieces and structural contact bonds produced by og.procgen.fracture.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `pieces` | `readonly FracturePiece[]` | `[]` | — | — | — | Closed shard meshes (hand each to og.render.bufferGeometry). |
| `bonds` | `readonly FractureBond[]` | `[]` | — | — | — | Contact bonds between adjacent shards. |

```ogt
struct FractureResult {
  doc "Solid pieces and structural contact bonds produced by og.procgen.fracture."
  field pieces List.FracturePiece "Closed shard meshes (hand each to og.render.bufferGeometry)."
  field bonds List.FractureBond "Contact bonds between adjacent shards."
}
```

<sub>Module `procgen` line 851</sub>
