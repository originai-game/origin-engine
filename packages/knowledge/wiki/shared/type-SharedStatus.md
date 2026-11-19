<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`shared`](../shared.md) · [index](../index.md)

<a id="type-SharedStatus"></a>

### `SharedStatus` (struct)

The session, as a HUD or the probe reads it.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `state` | `string` | `""` | — | — | — | `off`, `connecting`, `joined`, `reconnecting` or `refused`. |
| `host` | `string` | `""` | — | — | — | What carries the session here: the session provider's name (`static` for a fixed endpoint, `player` for the platform's), or `none`. |
| `transport` | `string` | `""` | — | — | — | The connection's transport: `ws` (WebSocket) or `wt` (WebTransport, snapshots as datagrams); empty without one. |
| `world` | `string` | `""` | — | — | — | The world joined. |
| `player` | `string` | `""` | — | — | — | This player's opaque id; empty before the welcome. |
| `name` | `string` | `""` | — | — | — | This player's display name. |
| `inSight` | `number` | `0` | — | — | — | The remote avatars held. |
| `others` | `number` | `0` | — | — | — | Visitors near this one the server does not send (past its 64 nearest). |
| `serverTick` | `number` | `0` | — | `ticks` | — | The newest server tick received. |
| `serverHz` | `number` | `0` | — | — | — | The server's tick rate. |
| `proposals` | `number` | `0` | — | — | — | Proposals sent. |
| `corrections` | `number` | `0` | — | — | — | Corrections received (og.SharedCorrected). |
| `bytesIn` | `number` | `0` | — | `bytes` | — | Bytes received in this session. |
| `bytesOut` | `number` | `0` | — | `bytes` | — | Bytes sent in this session. |
| `reason` | `string` | `""` | — | — | — | Why the last session ended or was refused; empty while none did. |

```ogt
struct SharedStatus {
  doc "The session, as a HUD or the probe reads it."
  field state String "`off`, `connecting`, `joined`, `reconnecting` or `refused`."
  field host String "What carries the session here: the session provider's name (`static` for a fixed endpoint, `player` for the platform's), or `none`."
  field transport String "The connection's transport: `ws` (WebSocket) or `wt` (WebTransport, snapshots as datagrams); empty without one."
  field world String "The world joined."
  field player String "This player's opaque id; empty before the welcome."
  field name String "This player's display name."
  field inSight u32 "The remote avatars held."
  field others u32 "Visitors near this one the server does not send (past its 64 nearest)."
  field serverTick f64 "The newest server tick received." {
    unit ticks
  }
  field serverHz u32 "The server's tick rate."
  field proposals f64 "Proposals sent."
  field corrections u32 "Corrections received (og.SharedCorrected)."
  field bytesIn f64 "Bytes received in this session." {
    unit bytes
  }
  field bytesOut f64 "Bytes sent in this session." {
    unit bytes
  }
  field reason String "Why the last session ended or was refused; empty while none did."
}
```

<sub>Module `shared` line 137</sub>
