<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`platform`](../platform.md) · [index](../index.md)

<a id="fn-storeGet"></a>

### `og.platform.storeGet`

Read one key of the player's store for this game.

Also known as: cloud load, read save data, key value read

```ts
og.platform.storeGet(world: World, key: string): Result<number>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `key` | `string` | no | The key. |

**Returns** `Result<number>`: Refused for a malformed key. The answer's data is `{ key, value, revision }` (value null and revision 0 for a key never written).

Binding symbol `og_platform_store_get` · publish builds: kept

```ogt
fn storeGet {
  doc "Read one key of the player's store for this game."
  aliases "cloud load, read save data, key value read"
  param world World "The world."
  param key String "The key."
  returns Result.u32 "Refused for a malformed key. The answer's data is `{ key, value, revision }` (value null and revision 0 for a key never written)."
}
```

<sub>Module `platform` line 120</sub>
