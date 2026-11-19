<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`shared`](../shared.md) · [index](../index.md)

<a id="fn-remove"></a>

### `og.shared.remove`

Remove a shared object near this visitor: its og.SharedObject entity despawns for everyone near it.

See `shared-worlds §5`.

```ts
og.shared.remove(world: World, key: string, expect: number | null): Result<number>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `key` | `string` | no | The object's key. |
| `expect` | `number \| null` | no | Only while it has this version; null: whatever it has. |

**Returns** `Result<number>`: The op's number (og.SharedOpAnswered.op). Refused without a joined session or for a key that is not one.

Binding symbol `og_shared_remove` · publish builds: kept

```ogt
fn remove {
  doc "Remove a shared object near this visitor: its og.SharedObject entity despawns for everyone near it."
  guide "shared-worlds §5"
  param world World "The world."
  param key String "The object's key."
  param expect Option.u32 "Only while it has this version; null: whatever it has."
  returns Result.u32 "The op's number (og.SharedOpAnswered.op). Refused without a joined session or for a key that is not one."
}
```

<sub>Module `shared` line 268</sub>
