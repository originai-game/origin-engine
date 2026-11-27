<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`shared`](../shared.md) · [index](../index.md)

<a id="record-SharedLeft"></a>

### `og.SharedLeft` (event)

The session ended: og.shared.leave, a refusal, or reconnecting gave up. Every og.RemoteAvatar despawns the same tick.

Token `og.SharedLeft`, registered under the schema name `og.SharedLeft`; its value type is `SharedLeft`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `reason` | `string` | `""` | — | — | — | Why: `left` (og.shared.leave), or a code and the provider's or the server's words: `unauthorized`, `full`, `version`, `shape`, `draining`, `unavailable` (no session provider here), `offline` (reconnecting gave up). |

```ogt
event SharedLeft {
  doc "The session ended: og.shared.leave, a refusal, or reconnecting gave up. Every og.RemoteAvatar despawns the same tick."
  field reason String "Why: `left` (og.shared.leave), or a code and the provider's or the server's words: `unauthorized`, `full`, `version`, `shape`, `draining`, `unavailable` (no session provider here), `offline` (reconnecting gave up)."
}
```

<sub>Module `shared` line 91</sub>
