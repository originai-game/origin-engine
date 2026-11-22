<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`load`](../load.md) · [index](../index.md)

<a id="fn-dataBytes"></a>

### `og.load.dataBytes`

The bytes of a ready og.load.data file read as `bytes`.

See `loading §14`.

```ts
og.load.dataBytes(world: World, asset: Asset): Result<Uint8Array>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world that owns the asset. |
| `asset` | `Asset` | no | A handle og.load.data answered. |

**Returns** `Result<Uint8Array>`: Refused while it is pending, when it failed (with its reason), when read as `json`, or for another kind of asset.

Binding symbol `og_load_data_bytes` · publish builds: kept

```ogt
fn dataBytes {
  custom true
  doc "The bytes of a ready og.load.data file read as `bytes`."
  guide "loading §14"
  param world World "The world that owns the asset."
  param asset Asset "A handle og.load.data answered."
  returns Result.Bytes "Refused while it is pending, when it failed (with its reason), when read as `json`, or for another kind of asset."
}
```

<sub>Module `load` line 694</sub>
