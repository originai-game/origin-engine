<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`load`](../load.md) · [index](../index.md)

<a id="fn-status"></a>

### `og.load.status`

Where a load is.

```ts
og.load.status(world: World, asset: Asset): LoadStatus
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world that owns the asset. |
| `asset` | `Asset` | no | The handle. |

**Returns** `LoadStatus`.

Binding symbol `og_load_status` · publish builds: kept

```ogt
fn status {
  doc "Where a load is."
  param world World "The world that owns the asset."
  param asset Asset "The handle."
  returns LoadStatus
}
```

<sub>`api.ogt` line 8271</sub>
