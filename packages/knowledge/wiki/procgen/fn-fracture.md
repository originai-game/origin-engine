<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`procgen`](../procgen.md) · [index](../index.md)

<a id="fn-fracture"></a>

### `og.procgen.fracture`

Fracture a closed mesh deterministically into solid Voronoi shards and contact bonds, returned as TypedArrays.

Also known as: voronoi fracture, shatter mesh, breakable chunks, destructible pieces

```ts
og.procgen.fracture(desc?: FractureDesc): Result<FractureResult>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `desc` | `FractureDesc` | yes | Input closed mesh buffers and fracture settings. |

**Returns** `Result<FractureResult>`: Refused for an open/zero-volume mesh, invalid indices or out-of-range settings.

Binding symbol `og_procgen_fracture` · publish builds: kept

```ogt
fn fracture {
  doc "Fracture a closed mesh deterministically into solid Voronoi shards and contact bonds, returned as TypedArrays."
  aliases "voronoi fracture, shatter mesh, breakable chunks, destructible pieces"
  param desc FractureDesc "Input closed mesh buffers and fracture settings."
  returns Result.FractureResult "Refused for an open/zero-volume mesh, invalid indices or out-of-range settings."
}
```

<sub>Module `procgen` line 857</sub>
