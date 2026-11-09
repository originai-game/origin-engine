<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-ChoreoStagger"></a>

### `ChoreoStagger` (struct)

One timeline on each target, each stepMs after the one before as ordered by from, the whole spread within capMs.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `timeline` | `string` | `""` | — | — | — | The timeline, by name. |
| `targets` | `readonly string[]` | `[]` | — | — | — | Paths below the play's element (empty: its children in order). |
| `stepMs` | `number` | `0` | — | `ms` | — | Between one target's start and the next's (0: the style's stagger). |
| `capMs` | `number` | `0` | — | `ms` | — | The most the whole stagger spreads over (0: the style's cap). |
| `from` | `string` | `"start"` | — | — | — | Where delays grow from: start, end, center (nearest the centre of their bounds first), or `x,y` (nearest that point first). |

```ogt
struct ChoreoStagger {
  doc "One timeline on each target, each stepMs after the one before as ordered by from, the whole spread within capMs."
  field timeline String "The timeline, by name."
  field targets List.String "Paths below the play's element (empty: its children in order)."
  field stepMs f32 "Between one target's start and the next's (0: the style's stagger)." {
    unit ms
  }
  field capMs f32 "The most the whole stagger spreads over (0: the style's cap)." {
    unit ms
  }
  field from String "Where delays grow from: start, end, center (nearest the centre of their bounds first), or `x,y` (nearest that point first)." {
    default "start"
  }
}
```

<sub>Module `ui` line 1226</sub>
