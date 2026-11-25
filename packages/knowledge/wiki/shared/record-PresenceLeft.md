<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`shared`](../shared.md) · [index](../index.md)

<a id="record-PresenceLeft"></a>

### `og.PresenceLeft` (event)

A visitor left sight (or the world): its og.RemoteAvatar entity was despawned this tick.

Token `og.PresenceLeft`, registered under the schema name `og.PresenceLeft`; its value type is `PresenceLeft`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `entity` | `Entity` | `0` | — | — | — | The entity it had. |
| `player` | `string` | `""` | — | — | — | Its opaque player id (at most 63 bytes). |

```ogt
event PresenceLeft {
  doc "A visitor left sight (or the world): its og.RemoteAvatar entity was despawned this tick."
  field entity Entity "The entity it had."
  field player String "Its opaque player id (at most 63 bytes)."
}
```

<sub>Module `shared` line 110</sub>
