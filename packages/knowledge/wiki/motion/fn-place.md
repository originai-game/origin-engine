<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`motion`](../motion.md) · [index](../index.md)

<a id="fn-place"></a>

### `og.motion.place`

Queue a library pose at a place and heading; needs og.MotionAgent.drive (the mover takes the Transform there).

```ts
og.motion.place(world: World, entity: Entity, desc?: PlaceGoalDesc): Result<void>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `entity` | `Entity` | no | An entity with an og.MotionAgent. |
| `desc` | `PlaceGoalDesc` | yes | The pose, place, heading and duration. |

**Returns** `Result<void>`: Refused for an entity without og.MotionAgent or without drive, a pose the library lacks, or a position that is not finite.

Binding symbol `og_motion_place` · publish builds: kept

```ogt
fn place {
  doc "Queue a library pose at a place and heading; needs og.MotionAgent.drive (the mover takes the Transform there)."
  param world World "The world."
  param entity Entity "An entity with an og.MotionAgent."
  param desc PlaceGoalDesc "The pose, place, heading and duration."
  returns Result.Void "Refused for an entity without og.MotionAgent or without drive, a pose the library lacks, or a position that is not finite."
}
```

<sub>Module `motion` line 261</sub>
