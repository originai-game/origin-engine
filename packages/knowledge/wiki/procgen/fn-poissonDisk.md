<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`procgen`](../procgen.md) · [index](../index.md)

<a id="fn-poissonDisk"></a>

### `og.procgen.poissonDisk`

Poisson-disk points in [0, width) × [0, height), in the order they were placed.

Also known as: blue noise, scatter points, even distribution, object placement, minimum distance sampling

```ts
og.procgen.poissonDisk(desc?: PoissonDesc): Result<readonly Vector2[]>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `desc` | `PoissonDesc` | yes | The area, the radius and the seed. |

**Returns** `Result<readonly Vector2[]>`: Refused for a size or radius that is not finite and above 0, or a radius so small the area needs more than 16,777,216 grid cells.

Binding symbol `og_procgen_poisson_disk` · publish builds: kept

```ogt
fn poissonDisk {
  doc "Poisson-disk points in [0, width) × [0, height), in the order they were placed."
  aliases "blue noise, scatter points, even distribution, object placement, minimum distance sampling"
  param desc PoissonDesc "The area, the radius and the seed."
  returns Result.List.Vector2 "Refused for a size or radius that is not finite and above 0, or a radius so small the area needs more than 16,777,216 grid cells."
}
```

<sub>Module `procgen` line 104</sub>
