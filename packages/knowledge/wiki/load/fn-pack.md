<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`load`](../load.md) · [index](../index.md)

<a id="fn-pack"></a>

### `og.load.pack`

Open a cooked bundle a content catalog's version lists by name (its manifest's `packs`), read from the content base against the integrity the version lists. Once ready it is a runtime bundle as og.load.bundle opens: og.load.bundleInfo, its groups and cells, og.load.release to close it.

Completion is host input, journaled as a `bundle` with its tick and manifest integrity, so a replay opens the same pack at the same tick. og.load.error starts with its reason, as og.load.data's. A pack opening or open answers its handle.

See `loading §14`.

```ts
og.load.pack(world: World, entry?: CatalogEntry): Result<Asset>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world that owns the asset. |
| `entry` | `CatalogEntry` | yes | The pack: its catalog, version, and its name as `path`. |

**Returns** `Result<Asset>`: The handle (pending), or why it was refused now: a malformed catalog, version or name.

Binding symbol `og_load_pack` · publish builds: kept

```ogt
fn pack {
  doc "Open a cooked bundle a content catalog's version lists by name (its manifest's `packs`), read from the content base against the integrity the version lists. Once ready it is a runtime bundle as og.load.bundle opens: og.load.bundleInfo, its groups and cells, og.load.release to close it."
  guide "loading §14"
  doc "Completion is host input, journaled as a `bundle` with its tick and manifest integrity, so a replay opens the same pack at the same tick. og.load.error starts with its reason, as og.load.data's. A pack opening or open answers its handle."
  param world World "The world that owns the asset."
  param entry CatalogEntry "The pack: its catalog, version, and its name as `path`."
  returns Result.Asset "The handle (pending), or why it was refused now: a malformed catalog, version or name."
}
```

<sub>Module `load` line 715</sub>
