<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`platform`](../platform.md) · [index](../index.md)

<a id="fn-storeSet"></a>

### `og.platform.storeSet`

Write one key of the player's store for this game; with a revision, only if nobody wrote it since (another device, another tab).

Also known as: cloud save, save data, persist key value, Steam Cloud write

```ts
og.platform.storeSet(world: World, desc?: StoreSetDesc): Result<number>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `desc` | `StoreSetDesc` | yes | The key, the value and the revision it must be at. |

**Returns** `Result<number>`: Refused for a malformed key or a value over 64 KiB. The answer's data is `{ key, revision }` (the new revision); `conflict` when the key moved past desc.revision.

Binding symbol `og_platform_store_set` · publish builds: kept

```ogt
fn storeSet {
  doc "Write one key of the player's store for this game; with a revision, only if nobody wrote it since (another device, another tab)."
  aliases "cloud save, save data, persist key value, Steam Cloud write"
  param world World "The world."
  param desc StoreSetDesc "The key, the value and the revision it must be at."
  returns Result.u32 "Refused for a malformed key or a value over 64 KiB. The answer's data is `{ key, revision }` (the new revision); `conflict` when the key moved past desc.revision."
}
```

<sub>Module `platform` line 126</sub>
