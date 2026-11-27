<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`combat`](../combat.md) · [index](../index.md)

<a id="record-AttackMove"></a>

### `og.AttackMove` (component)

One move's frame data, on an entity of its own that any og.Attack may play: its windows in ticks, its damage, the hit volumes it uses and the links out of it. All zero is no move: a link to it is never taken.

Also known as: attack data, move frame data, startup active recovery, hitbox timing, attack definition

Token `og.AttackMove`, registered under the schema name `og.AttackMove`; its value type is `AttackMove`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `startup` | `number` | `0` | ≥ 0 | — | tunable | Ticks before its active window. |
| `active` | `number` | `0` | ≥ 0 | — | tunable | Ticks its hit volumes strike. |
| `recovery` | `number` | `0` | ≥ 0 | — | tunable | Ticks after the active window before it ends. |
| `damage` | `number` | `0` | ≥ 0 | — | tunable | What a hit of its swing takes from the target's og.Health; 0 takes nothing. |
| `volumes` | `number` | `0` | — | — | — | Which of og.Attack.volumes strike in its active window, as bits (bit 0 is volumes[0]); 0 strikes with none. |
| `cancelOpen` | `number` | `0` | ≥ 0 | — | — | The first tick (from its start) of its cancel window, when og.Attack.cancelable holds. |
| `cancelClose` | `number` | `0` | ≥ 0 | — | — | The tick its cancel window closes; at or before cancelOpen, there is none. |
| `links` | `readonly Entity[]` | `[]` | — | — | a list of at most 8 `Entity` (`List.Entity.8`): reads a frozen copy, writes a whole array | The og.AttackLink entities followed out of it, in order: the first whose window holds the tick and whose action has a buffered press is taken. |

```ogt
component AttackMove {
  aliases "attack data, move frame data, startup active recovery, hitbox timing, attack definition"
  doc "One move's frame data, on an entity of its own that any og.Attack may play: its windows in ticks, its damage, the hit volumes it uses and the links out of it. All zero is no move: a link to it is never taken."
  field startup u32 "Ticks before its active window." {
    min 0
    tunable true
  }
  field active u32 "Ticks its hit volumes strike." {
    min 0
    tunable true
  }
  field recovery u32 "Ticks after the active window before it ends." {
    min 0
    tunable true
  }
  field damage f32 "What a hit of its swing takes from the target's og.Health; 0 takes nothing." {
    min 0
    tunable true
  }
  field volumes u32 "Which of og.Attack.volumes strike in its active window, as bits (bit 0 is volumes[0]); 0 strikes with none."
  field cancelOpen u32 "The first tick (from its start) of its cancel window, when og.Attack.cancelable holds." {
    min 0
  }
  field cancelClose u32 "The tick its cancel window closes; at or before cancelOpen, there is none." {
    min 0
  }
  field links List.Entity.8 "The og.AttackLink entities followed out of it, in order: the first whose window holds the tick and whose action has a buffered press is taken."
}
```

<sub>Module `combat` line 43</sub>
