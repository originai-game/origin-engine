<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`combat`](../combat.md) · [index](../index.md)

<a id="record-Projectile"></a>

### `og.Projectile` (component)

A shot's damage beside its og.HitVolume, without an og.Attack: leave the volume's `owner` 0, so each projectile strikes a target once. A hit takes `damage` under the target's og.Hurt, as a move's does. Only a strike that takes damage counts in `struck`: a target without og.Health (a graze volume), a dead one, one in its i-frames or its `shooter` lets it pass. Once `struck` reaches `pierce` og.systems.combat despawns it (with its descendants) before it strikes again. Moving it and a lifetime or range are the game's.

See `combat §2`.

Also known as: bullet, shot, missile, bullet damage, piercing shot, shmup bullet, Unity projectile, Godot bullet Area2D

Token `og.Projectile`, registered under the schema name `og.Projectile`; its value type is `Projectile`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `damage` | `number` | `0` | ≥ 0 | — | tunable | What each strike takes from the target's og.Health; 0 takes nothing, and is never spent. |
| `shooter` | `Entity` | `0` | — | — | may name a despawned entity | Who fired it: og.Damaged.source and og.Died.source (the projectile itself when 0), and never struck by it. It may be despawned while its shots fly (an enemy killed with its bullets in the air): they keep striking, and the events name it as it was. |
| `pierce` | `number` | `0` | ≥ 0 | — | tunable | Strikes that take damage before it is spent: 1 for a bullet that stops at the first it hurts, 0 for one never spent (a beam). |
| `struck` | `number` | `0` | — | — | read only | Written: its strikes that took damage. |

```ogt
component Projectile {
  aliases "bullet, shot, missile, bullet damage, piercing shot, shmup bullet, Unity projectile, Godot bullet Area2D"
  doc "A shot's damage beside its og.HitVolume, without an og.Attack: leave the volume's `owner` 0, so each projectile strikes a target once. A hit takes `damage` under the target's og.Hurt, as a move's does. Only a strike that takes damage counts in `struck`: a target without og.Health (a graze volume), a dead one, one in its i-frames or its `shooter` lets it pass. Once `struck` reaches `pierce` og.systems.combat despawns it (with its descendants) before it strikes again. Moving it and a lifetime or range are the game's."
  guide "combat §2"
  field damage f32 "What each strike takes from the target's og.Health; 0 takes nothing, and is never spent." {
    min 0
    tunable true
  }
  field shooter Entity "Who fired it: og.Damaged.source and og.Died.source (the projectile itself when 0), and never struck by it. It may be despawned while its shots fly (an enemy killed with its bullets in the air): they keep striking, and the events name it as it was." {
    weak true
  }
  field pierce u32 "Strikes that take damage before it is spent: 1 for a bullet that stops at the first it hurts, 0 for one never spent (a beam)." {
    min 0
    tunable true
  }
  field struck u32 "Written: its strikes that took damage." {
    readonly true
  }
}
```

<sub>Module `combat` line 89</sub>
