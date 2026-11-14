<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`net`](../net.md) · [index](../index.md)

<a id="fn-submit"></a>

### `og.net.submit`

Give this peer's player's input: in lockstep, for the tick inputDelayTicks ticks from the one running (sent to every peer when the tick ends); otherwise, for the next tick. It holds until the next call. Callable from a system.

See `networking §3`.

```ts
og.net.submit(world: World, input?: InputDesc): Result<void>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `input` | `InputDesc` | yes | The buttons and axes. |

**Returns** `Result<void>`: Refused for an axis that is not finite.

Binding symbol `og_net_submit` · publish builds: kept

```ogt
fn submit {
  doc "Give this peer's player's input: in lockstep, for the tick inputDelayTicks ticks from the one running (sent to every peer when the tick ends); otherwise, for the next tick. It holds until the next call. Callable from a system."
  guide "networking §3"
  param world World "The world."
  param input InputDesc "The buttons and axes."
  returns Result.Void "Refused for an axis that is not finite."
}
```

<sub>Module `net` line 196</sub>
