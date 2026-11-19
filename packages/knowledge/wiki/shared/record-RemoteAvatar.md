<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`shared`](../shared.md) · [index](../index.md)

<a id="record-RemoteAvatar"></a>

### `og.RemoteAvatar` (component)

Another visitor in sight, written by the engine: it spawns the entity with an og.Transform (and the game's avatar component, og.shared.shape) when the visitor comes into sight, moves it every tick, and despawns it when the visitor leaves. The game adds its own visuals to it. Who it is: og.shared.remote.

See `shared-worlds §3`.

Also known as: other visitors online, remote avatar, networked avatar, replicated entity, ghost, proxy

Token `og.RemoteAvatar`, registered under the schema name `og.RemoteAvatar`; its value type is `RemoteAvatar`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `id` | `number` | `0` | — | — | read only | This client's number for it, while it is in sight. |
| `kind` | `number` | `0` | — | — | read only | Its kind (og.SharedAvatar.kind). |
| `sinceTick` | `number` | `0` | — | `ticks` | read only | The tick it came into sight. |
| `jumpedTick` | `number` | `0` | — | `ticks` | read only | The tick it last jumped (a teleport: drawn there at once, not moved across); 0 never. |

```ogt
component RemoteAvatar {
  doc "Another visitor in sight, written by the engine: it spawns the entity with an og.Transform (and the game's avatar component, og.shared.shape) when the visitor comes into sight, moves it every tick, and despawns it when the visitor leaves. The game adds its own visuals to it. Who it is: og.shared.remote."
  aliases "other visitors online, remote avatar, networked avatar, replicated entity, ghost, proxy"
  guide "shared-worlds §3"
  field id u32 "This client's number for it, while it is in sight." {
    readonly true
  }
  field kind u32 "Its kind (og.SharedAvatar.kind)." {
    readonly true
  }
  field sinceTick f64 "The tick it came into sight." {
    readonly true
    unit ticks
  }
  field jumpedTick f64 "The tick it last jumped (a teleport: drawn there at once, not moved across); 0 never." {
    readonly true
    unit ticks
  }
}
```

<sub>Module `shared` line 60</sub>
