<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`play`](../play.md) · [index](../index.md)

<a id="record-HitEvent"></a>

### `og.HitEvent` (event)

A hit volume struck a target for the first time in its owner's swing, sent the tick it met it, in hit volume entity order, then where along its sweep, then target. A fact: what it does (damage, a stagger, a spark) is the game's.

Also known as: hit detected, on hit, attack landed, OnTriggerEnter hitbox

Token `og.HitEvent`, registered under the schema name `og.HitEvent`; its value type is `HitEvent`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `volume` | `Entity` | `0` | — | — | — | The og.HitVolume's entity. |
| `owner` | `Entity` | `0` | — | — | — | Its owner (og.HitVolume.owner, or the volume when that is 0). |
| `swing` | `number` | `0` | — | — | — | Its swing. |
| `target` | `Entity` | `0` | — | — | — | Who was struck: the og.HurtVolume's owner, or the collider's entity. |
| `hurt` | `Entity` | `0` | — | — | — | The og.HurtVolume met first; 0 for a collider. |
| `point` | `Vector3` | `{ x: 0, y: 0, z: 0 }` | — | `m` | worldspace `world` | Where they met, on the target's surface (inside it when the volume's core is). |
| `normal` | `Vector3` | `{ x: 0, y: 0, z: 0 }` | — | — | — | The direction from the target towards the volume there: a knockback's, a spark's. |

```ogt
event HitEvent {
  aliases "hit detected, on hit, attack landed, OnTriggerEnter hitbox"
  doc "A hit volume struck a target for the first time in its owner's swing, sent the tick it met it, in hit volume entity order, then where along its sweep, then target. A fact: what it does (damage, a stagger, a spark) is the game's."
  field volume Entity "The og.HitVolume's entity."
  field owner Entity "Its owner (og.HitVolume.owner, or the volume when that is 0)."
  field swing u32 "Its swing."
  field target Entity "Who was struck: the og.HurtVolume's owner, or the collider's entity."
  field hurt Entity "The og.HurtVolume met first; 0 for a collider."
  field point Vector3 "Where they met, on the target's surface (inside it when the volume's core is)." {
    unit m
    worldspace world
  }
  field normal Vector3 "The direction from the target towards the volume there: a knockback's, a spark's."
}
```

<sub>Module `play` line 92</sub>
