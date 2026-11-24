<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`probe`](../probe.md) · [index](../index.md)

<a id="type-SystemDump"></a>

### `SystemDump` (struct)

One system. The engine fills reads, writes and matched for every system the game declared; ms is null because the probe reads no clock (state().timings has the tick's time).

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `name` | `string` | `""` | — | — | — | Its name. |
| `matched` | `number \| null` | `null` | — | — | — | Entities its queries match now; null for a system the host did not declare to the probe. |
| `reads` | `readonly string[] \| null` | `null` | — | — | — | What it declared it reads; null when undeclared. |
| `writes` | `readonly string[] \| null` | `null` | — | — | — | What it declared it writes; null when undeclared. |
| `ms` | `number \| null` | `null` | — | `ms` | — | Time it took on the last tick; null when not measured. |
| `neverMatched` | `readonly string[] \| null` | `null` | — | — | optional | For a system that ran but never matched an entity: the smallest set of its required components no entity ever had together (empty when each set existed but its without or timing missed it). Null otherwise. og run and og test warn about each game system with one. |

```ogt
struct SystemDump {
  doc "One system. The engine fills reads, writes and matched for every system the game declared; ms is null because the probe reads no clock (state().timings has the tick's time)."
  field name String "Its name."
  field matched Option.u32 "Entities its queries match now; null for a system the host did not declare to the probe."
  field reads Option.List.String "What it declared it reads; null when undeclared."
  field writes Option.List.String "What it declared it writes; null when undeclared."
  field ms Option.f32 "Time it took on the last tick; null when not measured." {
    unit ms
  }
  field neverMatched Option.List.String "For a system that ran but never matched an entity: the smallest set of its required components no entity ever had together (empty when each set existed but its without or timing missed it). Null otherwise. og run and og test warn about each game system with one." {
    optional true
  }
}
```

<sub>Module `probe` line 431</sub>
