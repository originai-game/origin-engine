<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`platform`](../platform.md) · [index](../index.md)

<a id="fn-storeList"></a>

### `og.platform.storeList`

The keys of the player's store and their revisions.

```ts
og.platform.storeList(world: World): number
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |

**Returns** `number`: The answer's data is `{ keys: [{ key, revision }] }`.

Binding symbol `og_platform_store_list` · publish builds: kept

```ogt
fn storeList {
  doc "The keys of the player's store and their revisions."
  param world World "The world."
  returns u32 "The answer's data is `{ keys: [{ key, revision }] }`."
}
```

<sub>`api.ogt` line 15041</sub>
