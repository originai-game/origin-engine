<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`combat`](../combat.md) · [index](../index.md)

<a id="record-Hurt"></a>

### `og.Hurt` (component)

How a target takes a hit, beside its og.Health: for `invulnerableTicks` from a hit (its own tick first) further hits are ignored, for `staggerTicks` its og.Attack plays nothing (the move it played is cut), and og.Damaged carries a knockback of `knockback` away from the volume. A dodge's i-frames: write `invulnerableUntil`.

Also known as: invulnerability frames, i-frames, invincibility, hit stun, stagger, knockback, flinch

Token `og.Hurt`, registered under the schema name `og.Hurt`; its value type is `Hurt`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `invulnerableTicks` | `number` | `0` | ≥ 0 | — | tunable | Ticks from a hit in which further hits are ignored; 0 ignores none. |
| `invulnerableUntil` | `number` | `0` | ≥ 0 | — | — | The first tick it can be hurt again (hits read before it are ignored): written at a hit, and by a game for a dodge's i-frames. |
| `staggerTicks` | `number` | `0` | ≥ 0 | — | tunable | Ticks from a hit in which its og.Attack plays nothing; 0 staggers not at all. |
| `staggeredUntil` | `number` | `0` | ≥ 0 | — | — | The first tick its og.Attack plays again: written at a hit. |
| `knockback` | `number` | `0` | ≥ 0 | `mps` | tunable | The speed og.Damaged.knockback carries, away from the striking volume; 0 is none (applying it is the game's: a velocity, a controller's addVelocity). |

```ogt
component Hurt {
  aliases "invulnerability frames, i-frames, invincibility, hit stun, stagger, knockback, flinch"
  doc "How a target takes a hit, beside its og.Health: for `invulnerableTicks` from a hit (its own tick first) further hits are ignored, for `staggerTicks` its og.Attack plays nothing (the move it played is cut), and og.Damaged carries a knockback of `knockback` away from the volume. A dodge's i-frames: write `invulnerableUntil`."
  field invulnerableTicks u32 "Ticks from a hit in which further hits are ignored; 0 ignores none." {
    min 0
    tunable true
  }
  field invulnerableUntil u32 "The first tick it can be hurt again (hits read before it are ignored): written at a hit, and by a game for a dodge's i-frames." {
    min 0
  }
  field staggerTicks u32 "Ticks from a hit in which its og.Attack plays nothing; 0 staggers not at all." {
    min 0
    tunable true
  }
  field staggeredUntil u32 "The first tick its og.Attack plays again: written at a hit." {
    min 0
  }
  field knockback f32 "The speed og.Damaged.knockback carries, away from the striking volume; 0 is none (applying it is the game's: a velocity, a controller's addVelocity)." {
    min 0
    unit mps
    tunable true
  }
}
```

<sub>Module `combat` line 114</sub>
