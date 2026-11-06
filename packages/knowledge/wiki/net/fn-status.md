<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`net`](../net.md) · [index](../index.md)

<a id="fn-status"></a>

### `og.net.status`

Lockstep on this peer: running, the seats and who it waits for.

```ts
og.net.status(world: World): LockstepStatus
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |

**Returns** `LockstepStatus`.

Binding symbol `og_net_status` · publish builds: kept

```ogt
fn status {
  doc "Lockstep on this peer: running, the seats and who it waits for."
  param world World "The world."
  returns LockstepStatus
}
```

<sub>Module `net` line 233</sub>
