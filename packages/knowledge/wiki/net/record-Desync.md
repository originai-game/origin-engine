<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`net`](../net.md) · [index](../index.md)

<a id="record-Desync"></a>

### `og.Desync` (event)

This peer's world and another's differ: sent once, at the start of the tick after the hashes showed it. The probe's `desync` dump (`dump_desync`) names the component and field that differ, once the other peer's snapshot of that tick arrived.

Also known as: desync, out of sync, divergence, hash mismatch, simulation drift

Token `og.Desync`, registered under the schema name `og.Desync`; its value type is `Desync`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `tick` | `number` | `0` | — | — | — | The first divergent tick: the world after it differs (og.Time.tick of the snapshots compared). |
| `peer` | `number` | `0` | — | — | — | The first peer whose world differs from this one. |
| `peers` | `number` | `0` | — | — | — | How many peers' worlds differ from this one. |

```ogt
event Desync {
  doc "This peer's world and another's differ: sent once, at the start of the tick after the hashes showed it. The probe's `desync` dump (`dump_desync`) names the component and field that differ, once the other peer's snapshot of that tick arrived."
  aliases "desync, out of sync, divergence, hash mismatch, simulation drift"
  field tick u32 "The first divergent tick: the world after it differs (og.Time.tick of the snapshots compared)."
  field peer u32 "The first peer whose world differs from this one."
  field peers u32 "How many peers' worlds differ from this one."
}
```

<sub>Module `net` line 85</sub>
