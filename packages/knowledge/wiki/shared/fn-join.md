<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`shared`](../shared.md) · [index](../index.md)

<a id="fn-join"></a>

### `og.shared.join`

Join a shared world: the host asks its session provider for a connection (an endpoint and a token the engine never reads), the engine says hello, and og.SharedJoined arrives once the server takes this client in. The og.SharedAvatar entity is what it shares. A lost connection reconnects by itself.

See `shared-worlds §2`.

Also known as: connect to server, join online world, go online, enter MMO, start multiplayer session

```ts
og.shared.join(world: World, desc?: SharedJoinDesc): Result<void>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `desc` | `SharedJoinDesc` | yes | The world to join. |

**Returns** `Result<void>`: Refused during a session, with lockstep running (og.Lockstep), or for a world name that is not one. Without a session provider the join is refused when the host answers: og.SharedLeft says `unavailable`.

Binding symbol `og_shared_join` · publish builds: kept

```ogt
fn join {
  doc "Join a shared world: the host asks its session provider for a connection (an endpoint and a token the engine never reads), the engine says hello, and og.SharedJoined arrives once the server takes this client in. The og.SharedAvatar entity is what it shares. A lost connection reconnects by itself."
  aliases "connect to server, join online world, go online, enter MMO, start multiplayer session"
  guide "shared-worlds §2"
  param world World "The world."
  param desc SharedJoinDesc "The world to join."
  returns Result.Void "Refused during a session, with lockstep running (og.Lockstep), or for a world name that is not one. Without a session provider the join is refused when the host answers: og.SharedLeft says `unavailable`."
}
```

<sub>Module `shared` line 170</sub>
