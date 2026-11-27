<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`anim`](../anim.md) · [index](../index.md)

<a id="record-Hang"></a>

### `og.Hang` (component)

A hang on a rope: og.anim places the skeleton entity's og.Transform so the `grip` joint, as this tick's clips pose it, sits at `point`, its +Y running from the grip towards `anchor` by `lay` and its front (+Z) towards `facing`; then each of `limbs` (og.IkChain entities) gets its target on the rope, `reach` metres from the grip. After root motion, before og.FootLock and og.IkChain. One entity per hang; the game copies the rope's points in each tick.

Also known as: rope swing, hang from rope, grab ledge, hold bar, grip constraint, monkey bars, zipline

Token `og.Hang`, registered under the schema name `og.Hang`; its value type is `Hang`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `skeleton` | `Entity` | `0` | — | — | — | The entity with the og.Skeleton that hangs; it needs an og.Transform. |
| `grip` | `number` | `0` | ≥ 0 | — | — | The gripping joint (an index into the skin's joints): the hand on the rope. |
| `point` | `Vector3` | `{ x: 0, y: 0, z: 0 }` | — | `m` | worldspace `hook` | Where the grip is held, in the frame the skeleton entity's og.Transform is written in (world without a parent): the rope's end (og.physics.ropePoints' last). |
| `anchor` | `Vector3` | `{ x: 0, y: 0, z: 0 }` | — | `m` | worldspace `hook` | Where the rope is tied, in the same frame: the body lies along the line from here through `point`. |
| `facing` | `Vector3` | `{ x: 0, y: 0, z: 0 }` | — | — | — | The way the body's front (+Z) turns, in the same frame (0 -1 0: chest to the ground); the zero vector keeps the entity's. |
| `lay` | `number` | `1` | 0 … 1 | — | tunable | How far the body lies along the rope: 0 upright (its +Y up), 1 its +Y along the rope; ease it in and out as the swing starts and ends. |
| `weight` | `number` | `1` | 0 … 1 | — | tunable | How much of the hang is applied (0 = the entity's own Transform, 1 = hung). |
| `limbs` | `readonly Entity[]` | `[]` | — | — | a list of at most 4 `Entity` (`List.Entity.4`): reads a frozen copy, writes a whole array | og.IkChain entities on the same skeleton whose targets the hang sets each tick on the rope, in world space: the free hand, the feet. |
| `reach` | `readonly number[]` | `[]` | — | `m` | a list of at most 4 `f32` (`List.f32.4`): reads a frozen copy, writes a whole array | Per limb, its target's distance along the rope from the grip, towards the anchor above 0 (the other hand higher up), below it under 0; past the list's length 0. |
| `held` | `boolean` | `false` | — | — | read only | Written: the hang placed the skeleton this tick. |

```ogt
component Hang {
  doc "A hang on a rope: og.anim places the skeleton entity's og.Transform so the `grip` joint, as this tick's clips pose it, sits at `point`, its +Y running from the grip towards `anchor` by `lay` and its front (+Z) towards `facing`; then each of `limbs` (og.IkChain entities) gets its target on the rope, `reach` metres from the grip. After root motion, before og.FootLock and og.IkChain. One entity per hang; the game copies the rope's points in each tick."
  aliases "rope swing, hang from rope, grab ledge, hold bar, grip constraint, monkey bars, zipline"
  field skeleton Entity "The entity with the og.Skeleton that hangs; it needs an og.Transform."
  field grip u32 "The gripping joint (an index into the skin's joints): the hand on the rope." {
    min 0
  }
  field point Vector3 "Where the grip is held, in the frame the skeleton entity's og.Transform is written in (world without a parent): the rope's end (og.physics.ropePoints' last)." {
    worldspace hook
    unit m
  }
  field anchor Vector3 "Where the rope is tied, in the same frame: the body lies along the line from here through `point`." {
    worldspace hook
    unit m
  }
  field facing Vector3 "The way the body's front (+Z) turns, in the same frame (0 -1 0: chest to the ground); the zero vector keeps the entity's."
  field lay f32 "How far the body lies along the rope: 0 upright (its +Y up), 1 its +Y along the rope; ease it in and out as the swing starts and ends." {
    default 1
    min 0
    max 1
    tunable true
  }
  field weight f32 "How much of the hang is applied (0 = the entity's own Transform, 1 = hung)." {
    default 1
    min 0
    max 1
    tunable true
  }
  field limbs List.Entity.4 "og.IkChain entities on the same skeleton whose targets the hang sets each tick on the rope, in world space: the free hand, the feet."
  field reach List.f32.4 "Per limb, its target's distance along the rope from the grip, towards the anchor above 0 (the other hand higher up), below it under 0; past the list's length 0." {
    unit m
  }
  field held bool "Written: the hang placed the skeleton this tick." {
    readonly true
  }
}
```

<sub>Module `anim` line 480</sub>
