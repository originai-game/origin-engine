<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`shared`](../shared.md) · [index](../index.md)

<a id="record-SharedWorld"></a>

### `og.SharedWorld` (resource)

This client's shared-world session: its settings, which the game sets, and where it stands, which the engine writes. Remote avatars are drawn `interpDelayMs` behind the newest server time this client has, between the two states around it.

See `shared-worlds §2`.

Also known as: MMO, MMORPG, online presence, see other visitors, shared world, persistent world, open world online, visitors, avatars of others, server authoritative, interest management

Token `og.SharedWorld`, registered under the schema name `og.SharedWorld`; its value type is `SharedWorld`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `sendRateHz` | `number` | `20` | 1 … 60 | — | — | How often this client proposes its avatar's state, per second (at most the tick rate). |
| `interpDelayMs` | `number` | `100` | 0 … 1000 | `ms` | — | How far behind the newest server time remote avatars are drawn: two snapshots at the server's 20 Hz by default, so a lost one is covered. See `shared-worlds §3`. |
| `extrapolateMs` | `number` | `250` | 0 … 2000 | `ms` | — | How long a remote avatar whose updates stopped keeps moving along its last velocity before it holds. |
| `state` | `SharedState` | `"off"` | — | — | read only | Where the session stands. |
| `inSight` | `number` | `0` | — | — | read only | The remote avatars this client holds now (og.RemoteAvatar entities). |

```ogt
resource SharedWorld {
  doc "This client's shared-world session: its settings, which the game sets, and where it stands, which the engine writes. Remote avatars are drawn `interpDelayMs` behind the newest server time this client has, between the two states around it."
  aliases "MMO, MMORPG, online presence, see other visitors, shared world, persistent world, open world online, visitors, avatars of others, server authoritative, interest management"
  guide "shared-worlds §2"
  field sendRateHz u32 "How often this client proposes its avatar's state, per second (at most the tick rate)." {
    default 20
    min 1
    max 60
  }
  field interpDelayMs u32 "How far behind the newest server time remote avatars are drawn: two snapshots at the server's 20 Hz by default, so a lost one is covered." {
    guide "shared-worlds §3"
    default 100
    min 0
    max 1000
    unit ms
  }
  field extrapolateMs u32 "How long a remote avatar whose updates stopped keeps moving along its last velocity before it holds." {
    default 250
    min 0
    max 2000
    unit ms
  }
  field state SharedState "Where the session stands." {
    readonly true
  }
  field inSight u32 "The remote avatars this client holds now (og.RemoteAvatar entities)." {
    readonly true
  }
}
```

<sub>Module `shared` line 24</sub>
