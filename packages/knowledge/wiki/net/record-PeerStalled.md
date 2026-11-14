<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`net`](../net.md) · [index](../index.md)

<a id="record-PeerStalled"></a>

### `og.PeerStalled` (event)

A running peer has sent nothing for a second (or half of stallTimeoutMs, if shorter): for a "waiting for player" message. This peer's own (wall time decides it): never hashed, and a peer that speaks again is reported again the next time it falls silent.

Token `og.PeerStalled`, registered under the schema name `og.PeerStalled`; its value type is `PeerStalled`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `peer` | `number` | `0` | — | — | — | The silent peer. |
| `seat` | `number` | `0` | — | — | — | Its seat; og.Lockstep.players for a spectator. |
| `silentMs` | `number` | `0` | — | `ms` | — | How long it has been silent, in milliseconds. |

```ogt
event PeerStalled {
  doc "A running peer has sent nothing for a second (or half of stallTimeoutMs, if shorter): for a \"waiting for player\" message. This peer's own (wall time decides it): never hashed, and a peer that speaks again is reported again the next time it falls silent."
  field peer u32 "The silent peer."
  field seat u32 "Its seat; og.Lockstep.players for a spectator."
  field silentMs u32 "How long it has been silent, in milliseconds." {
    unit ms
  }
}
```

<sub>Module `net` line 111</sub>
