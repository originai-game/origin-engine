<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`shared`](../shared.md) · [index](../index.md)

<a id="fn-status"></a>

### `og.shared.status`

The session now: its state, what carries it, the avatars held, the server's tick, and the bytes and corrections so far.

```ts
og.shared.status(world: World): SharedStatus
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |

**Returns** `SharedStatus`.

Binding symbol `og_shared_status` · publish builds: kept

```ogt
fn status {
  doc "The session now: its state, what carries it, the avatars held, the server's tick, and the bytes and corrections so far."
  param world World "The world."
  returns SharedStatus
}
```

<sub>Module `shared` line 217</sub>
