<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`world`](../world.md) · [index](../index.md)

<a id="record-WorldRestored"></a>

### `og.WorldRestored` (event)

A world.restore applied: sent as the tick after it starts (after a mid-tick save, the first whole tick), not when the restore applies, so the world between the two hashes as the save did; read through world.events. The world came back as saved, request numbers and resources the session filled included; the session did not, so a system that reads this resets what belongs to the session.

See `saves §4`.

Token `og.WorldRestored`, registered under the schema name `og.WorldRestored`; its value type is `WorldRestored`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `from` | `number` | `0` | — | `ticks` | — | The tick the world was at when the restore applied. |
| `tick` | `number` | `0` | — | `ticks` | — | The saved tick: og.Time's tick right after the restore. |

```ogt
event WorldRestored {
  doc "A world.restore applied: sent as the tick after it starts (after a mid-tick save, the first whole tick), not when the restore applies, so the world between the two hashes as the save did; read through world.events. The world came back as saved, request numbers and resources the session filled included; the session did not, so a system that reads this resets what belongs to the session."
  guide "saves §4"
  field from u32 "The tick the world was at when the restore applied." {
    unit ticks
  }
  field tick u32 "The saved tick: og.Time's tick right after the restore." {
    unit ticks
  }
}
```

<sub>Module `world` line 186</sub>
