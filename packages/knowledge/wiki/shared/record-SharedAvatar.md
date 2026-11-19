<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`shared`](../shared.md) · [index](../index.md)

<a id="record-SharedAvatar"></a>

### `og.SharedAvatar` (component)

Marks the one entity whose state this client shares: its og.Transform's position and its yaw (the rotation about y), the velocity its moves imply, and the fields of the game's avatar component (og.shared.shape) on the same entity. The first in entity order is the one shared.

See `shared-worlds §2`.

Also known as: local player avatar, my character online, networked player, replicated player

Token `og.SharedAvatar`, registered under the schema name `og.SharedAvatar`; its value type is `SharedAvatar`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `kind` | `number` | `0` | — | — | — | The avatar's kind: which of the server's movement rules it moves by (its top speed). A token that names a kind wins. |

```ogt
component SharedAvatar {
  doc "Marks the one entity whose state this client shares: its og.Transform's position and its yaw (the rotation about y), the velocity its moves imply, and the fields of the game's avatar component (og.shared.shape) on the same entity. The first in entity order is the one shared."
  aliases "local player avatar, my character online, networked player, replicated player"
  guide "shared-worlds §2"
  field kind u32 "The avatar's kind: which of the server's movement rules it moves by (its top speed). A token that names a kind wins."
}
```

<sub>Module `shared` line 53</sub>
