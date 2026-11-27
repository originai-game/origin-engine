<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`platform`](../platform.md) · [index](../index.md)

<a id="fn-catalogHead"></a>

### `og.platform.catalogHead`

Which version of a content catalog is current: the one mutable thing about a catalog, whose versions never change once published (og.load.data reads a version's files). Poll it, every 30 to 60 seconds while the game cares: a catalog notice (og.PlatformNotice) is a hint a host may never send.

Also known as: content version, world index version, published level version, catalog pointer

```ts
og.platform.catalogHead(world: World, catalog: string): Result<number>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `catalog` | `string` | no | The catalog's name, as og.load.data's CatalogEntry.catalog. |

**Returns** `Result<number>`: Refused for a malformed name. The answer's data is `{ catalog, version, manifest, published }`: the version, its manifest's integrity (`sha384-<base64>`) and when it was published (UTC ms); `not_found` for a catalog never published.

Binding symbol `og_platform_catalog_head` · publish builds: kept

```ogt
fn catalogHead {
  doc "Which version of a content catalog is current: the one mutable thing about a catalog, whose versions never change once published (og.load.data reads a version's files). Poll it, every 30 to 60 seconds while the game cares: a catalog notice (og.PlatformNotice) is a hint a host may never send."
  aliases "content version, world index version, published level version, catalog pointer"
  param world World "The world."
  param catalog String "The catalog's name, as og.load.data's CatalogEntry.catalog."
  returns Result.u32 "Refused for a malformed name. The answer's data is `{ catalog, version, manifest, published }`: the version, its manifest's integrity (`sha384-<base64>`) and when it was published (UTC ms); `not_found` for a catalog never published."
}
```

<sub>Module `platform` line 139</sub>
