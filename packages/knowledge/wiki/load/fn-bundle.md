<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`load`](../load.md) · [index](../index.md)

<a id="fn-bundle"></a>

### `og.load.bundle`

Open another cooked bundle (the game's own `cooked` is open before the first tick). Its loads read through it once it is ready.

```ts
og.load.bundle(world: World, path: string): Result<Asset>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world that owns the asset. |
| `path` | `string` | no | The cooked directory, relative to the project (or the page), such as `dlc/cooked`. |

**Returns** `Result<Asset>`: The bundle's handle: pending until its manifest is read, then ready or failed.

Binding symbol `og_load_bundle` · publish builds: kept

```ogt
fn bundle {
  doc "Open another cooked bundle (the game's own `cooked` is open before the first tick). Its loads read through it once it is ready."
  param world World "The world that owns the asset."
  param path String "The cooked directory, relative to the project (or the page), such as `dlc/cooked`."
  returns Result.Asset "The bundle's handle: pending until its manifest is read, then ready or failed."
}
```

<sub>Module `load` line 185</sub>
