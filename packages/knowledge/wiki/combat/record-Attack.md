<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`combat`](../combat.md) · [index](../index.md)

<a id="record-Attack"></a>

### `og.Attack` (component)

An attacker's moves, played by og.systems.combat: `move` advances a tick each tick time moves (none in a hit stop) and ends after its startup, active and recovery ticks. While idle, a buffered press of a `starters` link's action starts its move; while a move plays, one of the move's links whose window holds `tick` does. Each start counts a `swing`, written with `active` and `owner` to the `volumes` the move's mask names.

See `combat §1`.

Also known as: attack state machine, combo state, melee attacker, fighting game moves, attack frames, cancel window

Token `og.Attack`, registered under the schema name `og.Attack`; its value type is `Attack`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `move` | `Entity` | `0` | — | — | — | The og.AttackMove playing (0: idle). Write 0 to stop it (a dodge out of a cancel window); start one with `next`. |
| `next` | `Entity` | `0` | — | — | — | An og.AttackMove to start at the next run whatever the windows (an AI's choice, a scripted move); cleared once taken. A staggered attacker drops it. |
| `tick` | `number` | `0` | — | — | read only | Written: ticks since the move started (0 its first). |
| `phase` | `AttackPhase` | `"idle"` | — | — | read only | Written: where in the move it is. |
| `swing` | `number` | `0` | — | — | read only | Written: how many moves it has started, written to its hit volumes' `swing`, so each move strikes a target once. |
| `swingMove` | `Entity` | `0` | — | — | read only | Written: the og.AttackMove the current swing started, kept after it ends or is cut: a hit is read the tick after it struck, and takes this move's damage. |
| `cancelable` | `boolean` | `false` | — | — | read only | Written: idle, or within the move's cancel window (og.AttackMove.cancelOpen to cancelClose): what a dodge or a jump checks before it writes `move` 0. |
| `starters` | `readonly Entity[]` | `[]` | — | — | a list of at most 8 `Entity` (`List.Entity.8`): reads a frozen copy, writes a whole array | The og.AttackLink entities taken while idle, in order (their windows are not read). |
| `volumes` | `readonly Entity[]` | `[]` | — | — | a list of at most 8 `Entity` (`List.Entity.8`): reads a frozen copy, writes a whole array | The og.HitVolume entities its moves drive: bit i of og.AttackMove.volumes is volumes[i]. |

```ogt
component Attack {
  aliases "attack state machine, combo state, melee attacker, fighting game moves, attack frames, cancel window"
  doc "An attacker's moves, played by og.systems.combat: `move` advances a tick each tick time moves (none in a hit stop) and ends after its startup, active and recovery ticks. While idle, a buffered press of a `starters` link's action starts its move; while a move plays, one of the move's links whose window holds `tick` does. Each start counts a `swing`, written with `active` and `owner` to the `volumes` the move's mask names."
  guide "combat §1"
  field move Entity "The og.AttackMove playing (0: idle). Write 0 to stop it (a dodge out of a cancel window); start one with `next`."
  field next Entity "An og.AttackMove to start at the next run whatever the windows (an AI's choice, a scripted move); cleared once taken. A staggered attacker drops it."
  field tick u32 "Written: ticks since the move started (0 its first)." {
    readonly true
  }
  field phase AttackPhase "Written: where in the move it is." {
    readonly true
  }
  field swing u32 "Written: how many moves it has started, written to its hit volumes' `swing`, so each move strikes a target once." {
    readonly true
  }
  field swingMove Entity "Written: the og.AttackMove the current swing started, kept after it ends or is cut: a hit is read the tick after it struck, and takes this move's damage." {
    readonly true
  }
  field cancelable bool "Written: idle, or within the move's cancel window (og.AttackMove.cancelOpen to cancelClose): what a dodge or a jump checks before it writes `move` 0." {
    readonly true
  }
  field starters List.Entity.8 "The og.AttackLink entities taken while idle, in order (their windows are not read)."
  field volumes List.Entity.8 "The og.HitVolume entities its moves drive: bit i of og.AttackMove.volumes is volumes[i]."
}
```

<sub>Module `combat` line 20</sub>
