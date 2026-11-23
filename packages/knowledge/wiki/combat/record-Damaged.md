<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`combat`](../combat.md) · [index](../index.md)

<a id="record-Damaged"></a>

### `og.Damaged` (event)

A hit took damage from a target's og.Health: sent by og.systems.combat the tick it read the og.HitEvent, in event order.

See `combat §2`.

Also known as: on damage, took damage, hit landed, damage event, damage number

Token `og.Damaged`, registered under the schema name `og.Damaged`; its value type is `Damaged`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `target` | `Entity` | `0` | — | — | — | The entity with the og.Health. |
| `source` | `Entity` | `0` | — | — | — | The attacker (og.HitEvent.owner), or the og.Projectile's shooter. |
| `move` | `Entity` | `0` | — | — | — | The og.AttackMove whose swing it was; 0 for an og.Projectile. |
| `swing` | `number` | `0` | — | — | — | The attacker's swing (og.HitEvent.swing). |
| `amount` | `number` | `0` | — | — | — | The damage taken (the move's or the projectile's, whatever was left). |
| `health` | `number` | `0` | — | — | — | What the target has left. |
| `point` | `Vector3` | `{ x: 0, y: 0, z: 0 }` | — | `m` | worldspace `world` | Where they met (og.HitEvent.point). |
| `knockback` | `Vector3` | `{ x: 0, y: 0, z: 0 }` | — | `mps` | — | og.Hurt.knockback along the hit, away from the volume: zero without og.Hurt or a knockback. |

```ogt
event Damaged {
  aliases "on damage, took damage, hit landed, damage event, damage number"
  doc "A hit took damage from a target's og.Health: sent by og.systems.combat the tick it read the og.HitEvent, in event order."
  guide "combat §2"
  field target Entity "The entity with the og.Health."
  field source Entity "The attacker (og.HitEvent.owner), or the og.Projectile's shooter."
  field move Entity "The og.AttackMove whose swing it was; 0 for an og.Projectile."
  field swing u32 "The attacker's swing (og.HitEvent.swing)."
  field amount f32 "The damage taken (the move's or the projectile's, whatever was left)."
  field health f32 "What the target has left."
  field point Vector3 "Where they met (og.HitEvent.point)." {
    unit m
    worldspace world
  }
  field knockback Vector3 "og.Hurt.knockback along the hit, away from the volume: zero without og.Hurt or a knockback." {
    unit mps
  }
}
```

<sub>Module `combat` line 143</sub>
