<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`shared`](../shared.md) · [index](../index.md)

<a id="fn-leave"></a>

### `og.shared.leave`

Leave the shared world: the server is told, and every og.RemoteAvatar despawns at the next tick (og.SharedLeft says `left`).

```ts
og.shared.leave(world: World): Result<void>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |

**Returns** `Result<void>`: Refused without a session.

Binding symbol `og_shared_leave` · publish builds: kept

```ogt
fn leave {
  doc "Leave the shared world: the server is told, and every og.RemoteAvatar despawns at the next tick (og.SharedLeft says `left`)."
  param world World "The world."
  returns Result.Void "Refused without a session."
}
```

<sub>Module `shared` line 217</sub>
