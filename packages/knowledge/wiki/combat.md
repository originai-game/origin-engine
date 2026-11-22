<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

# The `combat` module

The records og.systems.combat carries out: an og.Attack plays og.AttackMove entities tick by tick (startup, active, recovery), drives its og.HitVolume entities in the active window, and takes an og.AttackLink when its input action's buffered press lands in the link's window (og.input.consume); an og.HitEvent of a move's swing takes the move's damage from the target's og.Health, unless its og.Hurt says it is invulnerable, and is reported as og.Damaged and, once, og.Died. Every number is the game's: a zero window, damage or invulnerability does nothing.

See `combat §1`.

- **TypeScript**: functions are none at module level (its members live on the types below); components, resources and events are tokens on `og`.
- **Publish builds**: kept (members may override).
- **Index**: [every module and guide](index.md).

```ogt
module combat {
  doc "The records og.systems.combat carries out: an og.Attack plays og.AttackMove entities tick by tick (startup, active, recovery), drives its og.HitVolume entities in the active window, and takes an og.AttackLink when its input action's buffered press lands in the link's window (og.input.consume); an og.HitEvent of a move's swing takes the move's damage from the target's og.Health, unless its og.Hurt says it is invulnerable, and is reported as og.Damaged and, once, og.Died. Every number is the game's: a zero window, damage or invulnerability does nothing."
  # declarations follow, each on its own page
```

<sub>Module `combat` line 1</sub>

## Contents

| Name | Kind | Signature | Summary |
|---|---|---|---|
| [`AttackPhase`](combat/type-AttackPhase.md) | enum |  | Where an og.Attack is in its move. |
| [`AttackPhase()`](#column-AttackPhase) | column kind |  | An AttackPhase field: the variant's index. |
| [`og.Attack`](combat/record-Attack.md) | component |  | An attacker's moves, played by og.systems.combat: `move` advances a tick each tick time moves (none in a hit stop) and ends after its startup, active and recovery ticks. |
| [`og.AttackMove`](combat/record-AttackMove.md) | component |  | One move's frame data, on an entity of its own that any og.Attack may play: its windows in ticks, its damage, the hit volumes it uses and the links out of it. |
| [`og.AttackLink`](combat/record-AttackLink.md) | component |  | A way from one move to another: a press of `action` buffered (og.input.consume) while the move it leaves is at a tick in [open, close) starts `to`. |
| [`og.Health`](combat/record-Health.md) | component |  | What a target has left. |
| [`og.Hurt`](combat/record-Hurt.md) | component |  | How a target takes a hit, beside its og.Health: for `invulnerableTicks` from a hit (its own tick first) further hits are ignored, for `staggerTicks` its og.Attack plays nothing (the move it played is… |
| [`og.Damaged`](combat/record-Damaged.md) | event |  | A hit took damage from a target's og.Health: sent by og.systems.combat the tick it read the og.HitEvent, in event order. |
| [`og.Died`](combat/record-Died.md) | event |  | A target's og.Health reached 0: sent once, the tick the hit was read (og.Health.dead is set). |
| [`og.CombatAttackers`](combat/record-CombatAttackers.md) | resource |  | Every og.Attack entity in entity order, written by og.systems.combat's first system each tick for its second, which reads the others' records entity by entity (at most 1,024 attackers). |

## Column kinds

Each is a factory, `AttackPhase(value?, options?)`, that makes one field of a component or resource layout.

| Column | Value | Typed array | Stride | Doc |
|---|---|---|---|---|
| <a id="column-AttackPhase"></a>`AttackPhase` | `AttackPhase` | `Uint32Array` | 1 | An AttackPhase field: the variant's index. |

```ogt
column AttackPhase {
  doc "An AttackPhase field: the variant's index."
  array Uint32Array
  stride 1
  value AttackPhase
}
```

<sub>Module `combat` line 13</sub>
