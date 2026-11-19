<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`shared`](../shared.md) · [index](../index.md)

<a id="record-PresenceEntered"></a>

### `og.PresenceEntered` (event)

A visitor came into sight: its og.RemoteAvatar entity exists from this tick.

Token `og.PresenceEntered`, registered under the schema name `og.PresenceEntered`; its value type is `PresenceEntered`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `entity` | `Entity` | `0` | — | — | — | Its og.RemoteAvatar entity. |
| `player` | `string` | `""` | — | — | — | Its opaque player id (at most 63 bytes). |
| `name` | `string` | `""` | — | — | — | Its display name (at most 63 bytes). |
| `kind` | `number` | `0` | — | — | — | Its kind. |

```ogt
event PresenceEntered {
  doc "A visitor came into sight: its og.RemoteAvatar entity exists from this tick."
  field entity Entity "Its og.RemoteAvatar entity."
  field player String "Its opaque player id (at most 63 bytes)."
  field name String "Its display name (at most 63 bytes)."
  field kind u32 "Its kind."
}
```

<sub>Module `shared` line 92</sub>
