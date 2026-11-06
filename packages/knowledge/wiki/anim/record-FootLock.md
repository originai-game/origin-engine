<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`anim`](../anim.md) · [index](../index.md)

<a id="record-FootLock"></a>

### `og.FootLock` (component)

Foot locking: while the playing clips plant `foot` (within 5 mm of its lowest in each clip, the notion retargeting uses, weighed across blends and cross-fades: planted when the planting clips hold at least half the pose), the foot stays where it touched the ground, through clip blends and root motion, its leg bent by two-bone IK and the foot keeping its animated orientation; when the clips lift it, the foot rejoins its animation over `release`. Put it on its own entity, one per foot; locks apply after root motion and before og.IkChain, in query order, and og.JointLimit clamps the leg. The lock is world state (the `og.anim` state block).

Token `og.FootLock`, registered under the schema name `og.FootLock`; its value type is `FootLock`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `skeleton` | `Entity` | `0` | — | — | — | The entity with the og.Skeleton whose foot this holds. |
| `foot` | `number` | `0` | ≥ 0 | — | — | The foot joint (an index into the skin's joints); the two bones above it are the leg that bends. |
| `weight` | `number` | `1` | 0 … 1 | — | tunable | How much of the lock is applied (0 = the animated foot, 1 = held). |
| `release` | `number` | `0.15` | 0 … 2 | `s` | tunable | Seconds the foot takes to rejoin its animation once lifted (0: at once). |
| `maxDistance` | `number` | `0.5` | ≥ 0 | `m` | tunable | Farther than this from where the animation puts the foot, the lock lets go (a leg cannot stretch). |
| `planted` | `boolean` | `false` | — | — | read only | Written: the playing clips plant the foot this tick. |
| `locked` | `boolean` | `false` | — | — | read only | Written: the foot is held at its lock this tick. |

```ogt
component FootLock {
  doc "Foot locking: while the playing clips plant `foot` (within 5 mm of its lowest in each clip, the notion retargeting uses, weighed across blends and cross-fades: planted when the planting clips hold at least half the pose), the foot stays where it touched the ground, through clip blends and root motion, its leg bent by two-bone IK and the foot keeping its animated orientation; when the clips lift it, the foot rejoins its animation over `release`. Put it on its own entity, one per foot; locks apply after root motion and before og.IkChain, in query order, and og.JointLimit clamps the leg. The lock is world state (the `og.anim` state block)."
  field skeleton Entity "The entity with the og.Skeleton whose foot this holds."
  field foot u32 "The foot joint (an index into the skin's joints); the two bones above it are the leg that bends." {
    min 0
  }
  field weight f32 "How much of the lock is applied (0 = the animated foot, 1 = held)." {
    default 1
    min 0
    max 1
    tunable true
  }
  field release f32 "Seconds the foot takes to rejoin its animation once lifted (0: at once)." {
    default 0.15
    min 0
    max 2
    unit s
    tunable true
  }
  field maxDistance f32 "Farther than this from where the animation puts the foot, the lock lets go (a leg cannot stretch)." {
    default 0.5
    min 0
    unit m
    tunable true
  }
  field planted bool "Written: the playing clips plant the foot this tick." {
    readonly true
  }
  field locked bool "Written: the foot is held at its lock this tick." {
    readonly true
  }
}
```

<sub>Module `anim` line 396</sub>
