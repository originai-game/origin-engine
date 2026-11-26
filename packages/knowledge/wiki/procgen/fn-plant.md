<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`procgen`](../procgen.md) · [index](../index.md)

<a id="fn-plant"></a>

### `og.procgen.plant`

Grow a deterministic procedural tree, shrub or flower into GPU-ready TypedArray mesh buffers (pass directly to og.render.bufferGeometry).

Also known as: procedural tree, generate plant, foliage mesh, ez-tree, bush

```ts
og.procgen.plant(desc?: PlantDesc): Result<PlantMeshResult>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `desc` | `PlantDesc` | yes | Growth habit, branch levels, leaves and flowers. |

**Returns** `Result<PlantMeshResult>`: Refused if parameters are invalid or exceed branch/leaf limits.

Binding symbol `og_procgen_plant` · publish builds: kept

```ogt
fn plant {
  doc "Grow a deterministic procedural tree, shrub or flower into GPU-ready TypedArray mesh buffers (pass directly to og.render.bufferGeometry)."
  aliases "procedural tree, generate plant, foliage mesh, ez-tree, bush"
  param desc PlantDesc "Growth habit, branch levels, leaves and flowers."
  returns Result.PlantMeshResult "Refused if parameters are invalid or exceed branch/leaf limits."
}
```

<sub>Module `procgen` line 861</sub>
