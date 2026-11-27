<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`abilities`](../abilities.md) · [index](../index.md)

<a id="record-Status"></a>

### `og.Status` (component)

An effect on a target, its parent (og.Parent): spawned there to apply it. og.systems.abilities merges it with the target's status of the same kind by that status's `stacking`, counts down `ticks` each tick time moves (its first tick counting) and despawns it, and its modifiers, when they run out.

Also known as: status effect, buff, debuff, poison, burn, slow, haste, stun, shield, damage over time, aura

Token `og.Status`, registered under the schema name `og.Status`; its value type is `Status`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `kind` | `string` | `""` | — | — | `str(32)`: at most 32 bytes of UTF-8 | Which statuses on one target are one effect; empty never merges. |
| `stacking` | `StatusStacking` | `"refresh"` | — | — | — | What one of the same kind applied after it does. |
| `ticks` | `number` | `0` | ≥ 0 | `ticks` | tunable | Ticks left; 0 lasts until it is despawned. |
| `stacks` | `number` | `0` | — | — | — | How many stacks it holds: each of its modifiers counts this many times; 0 changes nothing. |
| `maxStacks` | `number` | `0` | ≥ 0 | — | tunable | The most stacks `stack` adds up to; 0 adds none. |
| `strength` | `number` | `0` | — | — | — | What `strongest` compares. |
| `applied` | `boolean` | `false` | — | — | read only | Written: it has been merged (or found alone) on its target. |

```ogt
component Status {
  aliases "status effect, buff, debuff, poison, burn, slow, haste, stun, shield, damage over time, aura"
  doc "An effect on a target, its parent (og.Parent): spawned there to apply it. og.systems.abilities merges it with the target's status of the same kind by that status's `stacking`, counts down `ticks` each tick time moves (its first tick counting) and despawns it, and its modifiers, when they run out."
  field kind str.32 "Which statuses on one target are one effect; empty never merges."
  field stacking StatusStacking "What one of the same kind applied after it does."
  field ticks u32 "Ticks left; 0 lasts until it is despawned." {
    min 0
    unit ticks
    tunable true
  }
  field stacks u32 "How many stacks it holds: each of its modifiers counts this many times; 0 changes nothing."
  field maxStacks u32 "The most stacks `stack` adds up to; 0 adds none." {
    min 0
    tunable true
  }
  field strength f32 "What `strongest` compares."
  field applied bool "Written: it has been merged (or found alone) on its target." {
    readonly true
  }
}
```

<sub>Module `abilities` line 91</sub>
