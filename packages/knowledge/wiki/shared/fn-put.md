<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`shared`](../shared.md) · [index](../index.md)

<a id="fn-put"></a>

### `og.shared.put`

Create or change a shared object near this visitor: the server applies it in the order ops arrive, and every visitor near it (this one too) sees its og.SharedObject entity change; og.SharedOpAnswered says whether it was taken. Kept across server restarts when the server keeps a store.

```ts
og.shared.put<C extends AnyComponent>(world: World, key: string, component: C, value: Partial<Value<C>>, desc?: SharedPutDesc): Result<number>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `key` | `string` | no | The object's key in this world: 1 to 64 of letters, digits, `.`, `_`, `-`, `:` and `/` (`door/12`). |
| `component` | `C` | no | The object component og.shared.shape named. |
| `value` | `Partial<Value<C>>` | no | Its fields; omitted ones take their defaults. |
| `desc` | `SharedPutDesc` | yes | Where it is, and the version it expects. |

**Returns** `Result<number>`: The op's number (og.SharedOpAnswered.op). Refused without a joined session, without an object shape, for another component, or a key that is not one.

Binding symbol `og_shared_put` · publish builds: kept

```ogt
fn put {
  doc "Create or change a shared object near this visitor: the server applies it in the order ops arrive, and every visitor near it (this one too) sees its og.SharedObject entity change; og.SharedOpAnswered says whether it was taken. Kept across server restarts when the server keeps a store."
  generic C AnyComponent
  param world World "The world."
  param key String "The object's key in this world: 1 to 64 of letters, digits, `.`, `_`, `-`, `:` and `/` (`door/12`)."
  param component C "The object component og.shared.shape named."
  param value Partial.Value.C "Its fields; omitted ones take their defaults."
  param desc SharedPutDesc "Where it is, and the version it expects."
  returns Result.u32 "The op's number (og.SharedOpAnswered.op). Refused without a joined session, without an object shape, for another component, or a key that is not one."
}
```

<sub>Module `shared` line 247</sub>
