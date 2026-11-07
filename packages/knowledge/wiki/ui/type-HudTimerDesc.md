<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-HudTimerDesc"></a>

### `HudTimerDesc` (desc struct)

A round, match or turn timer.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `seconds` | `number` | `0` | — | `s` | — | The time shown. |
| `total` | `number` | `0` | ≥ 0 | `s` | — | A countdown's whole length (for its ring); 0 draws no ring. |
| `mode` | `TimerMode` | `"countdown"` | — | — | — | Which way it runs. |
| `label` | `string` | `""` | — | — | — | A label over it (Round 2); empty is none. |
| `warnAt` | `number` | `10` | — | `s` | — | A countdown under this many seconds warns, then pulses in danger under a third of it. |
| `tenths` | `boolean` | `true` | — | — | — | Tenths under ten seconds (the default). |
| `paused` | `boolean` | `false` | — | — | — | Paused: it dims and blinks. |
| `anchor` | `Anchor \| null` | `"top"` | — | — | — | Opens its own anchored group there; null builds it in place. |

```ogt
struct HudTimerDesc {
  doc "A round, match or turn timer."
  desc true
  field seconds f32 "The time shown." {
    unit s
  }
  field total f32 "A countdown's whole length (for its ring); 0 draws no ring." {
    min 0
    unit s
  }
  field mode TimerMode "Which way it runs."
  field label String "A label over it (Round 2); empty is none."
  field warnAt f32 "A countdown under this many seconds warns, then pulses in danger under a third of it." {
    default 10
    unit s
  }
  field tenths bool "Tenths under ten seconds (the default)." {
    default true
  }
  field paused bool "Paused: it dims and blinks."
  field anchor Option.Anchor "Opens its own anchored group there; null builds it in place." {
    default top
  }
}
```

<sub>Module `ui` line 6353</sub>
