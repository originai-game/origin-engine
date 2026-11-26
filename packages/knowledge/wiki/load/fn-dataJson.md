<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`load`](../load.md) · [index](../index.md)

<a id="fn-dataJson"></a>

### `og.load.dataJson`

The value of a ready og.load.data file read as `json`.

See `loading §14`.

```ts
og.load.dataJson(world: World, asset: Asset): Result<unknown>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world that owns the asset. |
| `asset` | `Asset` | no | A handle og.load.data answered. |

**Returns** `Result<unknown>`: Refused while it is pending, when it failed (with its reason), when read as `bytes`, or for another kind of asset.

Binding symbol `og_load_data_json` · publish builds: kept

```ogt
fn dataJson {
  doc "The value of a ready og.load.data file read as `json`."
  guide "loading §14"
  param world World "The world that owns the asset."
  param asset Asset "A handle og.load.data answered."
  returns Result.Json "Refused while it is pending, when it failed (with its reason), when read as `bytes`, or for another kind of asset."
}
```

<sub>Module `load` line 722</sub>
