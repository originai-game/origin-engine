<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`load`](../load.md) · [index](../index.md)

<a id="fn-environmentInfo"></a>

### `og.load.environmentInfo`

A ready uncooked environment's map and sizes.

```ts
og.load.environmentInfo(world: World, asset: Asset): Result<EnvironmentInfo>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world that owns the asset. |
| `asset` | `Asset` | no | A handle og.load.environmentData answered. |

**Returns** `Result<EnvironmentInfo>`: Refused while it is pending (say so), when it failed (with its reason) or for another kind of asset.

Binding symbol `og_load_environment_info` · publish builds: kept

```ogt
fn environmentInfo {
  doc "A ready uncooked environment's map and sizes."
  param world World "The world that owns the asset."
  param asset Asset "A handle og.load.environmentData answered."
  returns Result.EnvironmentInfo "Refused while it is pending (say so), when it failed (with its reason) or for another kind of asset."
}
```

<sub>`api.ogt` line 8458</sub>
