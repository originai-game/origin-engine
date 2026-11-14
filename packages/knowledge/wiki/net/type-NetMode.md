<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`net`](../net.md) · [index](../index.md)

<a id="type-NetMode"></a>

### `NetMode` (enum)

How a room's peers take their ticks (og.Lockstep.mode).

A string: one of the variants below. The zero value is `"lockstep"`.

| Variant | Doc |
|---|---|
| `"lockstep"` | Each tick waits until every seated player's input for it is in: no prediction, no rollback; a late input holds the world. |
| `"rollback"` | A tick runs on a missing input predicted as the player's last, up to maxPredictionTicks past the last tick whose inputs are all in; when the real input differs the world is rewound to its snapshot and the ticks re-run within the frame (GGPO-style). Hashes and og.Desync cover the confirmed world. |

```ogt
enum NetMode {
  doc "How a room's peers take their ticks (og.Lockstep.mode)."
  variant lockstep "Each tick waits until every seated player's input for it is in: no prediction, no rollback; a late input holds the world."
  variant rollback "A tick runs on a missing input predicted as the player's last, up to maxPredictionTicks past the last tick whose inputs are all in; when the real input differs the world is rewound to its snapshot and the ticks re-run within the frame (GGPO-style). Hashes and og.Desync cover the confirmed world."
}
```

<sub>Module `net` line 7</sub>
