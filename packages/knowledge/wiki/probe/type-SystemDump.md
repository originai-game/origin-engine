<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`probe`](../probe.md) · [index](../index.md)

<a id="type-SystemDump"></a>

### `SystemDump` (struct)

One system: declared access, matched count, and timing (ms, totalMs, runs) when the host supplies a clock.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `name` | `string` | `""` | — | — | — | Its name. |
| `matched` | `number \| null` | `null` | — | — | — | Entities its queries match now; null for a system the host did not declare to the probe. |
| `reads` | `readonly string[] \| null` | `null` | — | — | — | What it declared it reads; null when undeclared. |
| `writes` | `readonly string[] \| null` | `null` | — | — | — | What it declared it writes; null when undeclared. |
| `ms` | `number \| null` | `null` | — | `ms` | — | Time it took on the last tick; null when not measured. |
| `totalMs` | `number \| null` | `null` | — | `ms` | optional | Cumulative milliseconds across timed runs; null when not measured. |
| `runs` | `number` | `0` | — | — | optional | Number of timed runs accumulated in totalMs. |
| `neverMatched` | `readonly string[] \| null` | `null` | — | — | optional | For a system that ran but never matched an entity: the smallest set of its required components no entity ever had together (empty when each set existed but its without or timing missed it). Null otherwise. og run and og test warn about each game system with one. |

```ogt
struct SystemDump {
  doc "One system: declared access, matched count, and timing (ms, totalMs, runs) when the host supplies a clock."
  field name String "Its name."
  field matched Option.u32 "Entities its queries match now; null for a system the host did not declare to the probe."
  field reads Option.List.String "What it declared it reads; null when undeclared."
  field writes Option.List.String "What it declared it writes; null when undeclared."
  field ms Option.f32 "Time it took on the last tick; null when not measured." {
    unit ms
  }
  field totalMs Option.f32 "Cumulative milliseconds across timed runs; null when not measured." {
    optional true
    unit ms
  }
  field runs u32 "Number of timed runs accumulated in totalMs." {
    optional true
  }
  field neverMatched Option.List.String "For a system that ran but never matched an entity: the smallest set of its required components no entity ever had together (empty when each set existed but its without or timing missed it). Null otherwise. og run and og test warn about each game system with one." {
    optional true
  }
}
```

<sub>Module `probe` line 442</sub>
