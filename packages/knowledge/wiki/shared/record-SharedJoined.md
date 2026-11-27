<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`shared`](../shared.md) · [index](../index.md)

<a id="record-SharedJoined"></a>

### `og.SharedJoined` (event)

The server took this client in: sent at the start of the tick after its welcome arrived, and again after a reconnect.

Token `og.SharedJoined`, registered under the schema name `og.SharedJoined`; its value type is `SharedJoined`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `player` | `string` | `""` | — | — | — | This player's opaque id, the provider's (at most 63 bytes). |
| `name` | `string` | `""` | — | — | — | This player's display name (at most 63 bytes). |
| `resumed` | `boolean` | `false` | — | — | — | Whether the server resumed this avatar from before a lost connection (it stayed where it was for the others). |

```ogt
event SharedJoined {
  doc "The server took this client in: sent at the start of the tick after its welcome arrived, and again after a reconnect."
  field player String "This player's opaque id, the provider's (at most 63 bytes)."
  field name String "This player's display name (at most 63 bytes)."
  field resumed bool "Whether the server resumed this avatar from before a lost connection (it stayed where it was for the others)."
}
```

<sub>Module `shared` line 84</sub>
