<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`load`](../load.md) · [index](../index.md)

<a id="fn-bundle"></a>

### `og.load.bundle`

Open another cooked bundle and register its streaming groups, including cell groups. Loads and group residency use that bundle once ready. A group name already declared by another open bundle fails the open, naming both bundles.

Scene, sub-scene and model dependencies resolve from their own bundle first, with loads cached per bundle and path. An id absent there falls back to the boot bundle, then ready bundles in open order. Direct og.load calls use that fallback order.

See `world §4`.

```ts
og.load.bundle(world: World, path: string): Result<Asset>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world that owns the asset. |
| `path` | `string` | no | The cooked directory, relative to the project (or the page), such as `dlc/cooked`. |

**Returns** `Result<Asset>`: The bundle's handle: ready or failed at the call when the host has its manifest (natively always), else pending until a tick boundary. og.load.completions journals the tick, so a replay opens it at the same one.

Binding symbol `og_load_bundle` · publish builds: kept

```ogt
fn bundle {
  doc "Open another cooked bundle and register its streaming groups, including cell groups. Loads and group residency use that bundle once ready. A group name already declared by another open bundle fails the open, naming both bundles."
  doc "Scene, sub-scene and model dependencies resolve from their own bundle first, with loads cached per bundle and path. An id absent there falls back to the boot bundle, then ready bundles in open order. Direct og.load calls use that fallback order."
  guide "world §4"
  param world World "The world that owns the asset."
  param path String "The cooked directory, relative to the project (or the page), such as `dlc/cooked`."
  returns Result.Asset "The bundle's handle: ready or failed at the call when the host has its manifest (natively always), else pending until a tick boundary. og.load.completions journals the tick, so a replay opens it at the same one."
}
```

<sub>Module `load` line 185</sub>
