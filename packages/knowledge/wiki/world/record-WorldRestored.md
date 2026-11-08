<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`world`](../world.md) · [index](../index.md)

<a id="record-WorldRestored"></a>

### `og.WorldRestored` (event)

A world.restore applied: the tick after it (the first whole tick after a save taken mid-tick, whose next step completes the saved tick) delivers this, read through world.events like any event. Everything in the world came back as saved, request numbers and resources the session filled included; the session did not, so a system that reads this resets what belongs to the session: it forgets requests answered long ago, asks the store again for the settings and progress, and closes a menu the player had open. Sent as that tick starts, not when the restore applies, so the world between the two hashes as the save did.

Token `og.WorldRestored`, registered under the schema name `og.WorldRestored`; its value type is `WorldRestored`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `from` | `number` | `0` | — | `ticks` | — | The tick the world was at when the restore applied. |
| `tick` | `number` | `0` | — | `ticks` | — | The saved tick: og.Time's tick right after the restore. |

```ogt
event WorldRestored {
  doc "A world.restore applied: the tick after it (the first whole tick after a save taken mid-tick, whose next step completes the saved tick) delivers this, read through world.events like any event. Everything in the world came back as saved, request numbers and resources the session filled included; the session did not, so a system that reads this resets what belongs to the session: it forgets requests answered long ago, asks the store again for the settings and progress, and closes a menu the player had open. Sent as that tick starts, not when the restore applies, so the world between the two hashes as the save did."
  field from u32 "The tick the world was at when the restore applied." {
    unit ticks
  }
  field tick u32 "The saved tick: og.Time's tick right after the restore." {
    unit ticks
  }
}
```

<sub>Module `world` line 177</sub>
