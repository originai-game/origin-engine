<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`motion`](../motion.md) · [index](../index.md)

<a id="fn-walkTo"></a>

### `og.motion.walkTo`

Queue a walk (or a run, above 2.5 m/s) to a point; needs og.MotionAgent.drive. The mover travels in a straight line up to speed, brakes at deceleration, and turns at turnSpeed. With a CharacterController it writes the controller's move input before physics; collision and ground height remain the controller's. The goal ends on actual arrival and completion of arrivalHeading, when supplied.

Also known as: walk to, move to, go to, locomotion goal, natural movement, Unreal MoveTo

```ts
og.motion.walkTo(world: World, entity: Entity, desc?: WalkGoalDesc): Result<void>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `entity` | `Entity` | no | An entity with an og.MotionAgent. |
| `desc` | `WalkGoalDesc` | yes | Where, how fast, and the pose on arrival. |

**Returns** `Result<void>`: Refused for an entity without og.MotionAgent or without drive, or a position that is not finite.

Binding symbol `og_motion_walk_to` · publish builds: kept

```ogt
fn walkTo {
  doc "Queue a walk (or a run, above 2.5 m/s) to a point; needs og.MotionAgent.drive. The mover travels in a straight line up to speed, brakes at deceleration, and turns at turnSpeed. With a CharacterController it writes the controller's move input before physics; collision and ground height remain the controller's. The goal ends on actual arrival and completion of arrivalHeading, when supplied."
  aliases "walk to, move to, go to, locomotion goal, natural movement, Unreal MoveTo"
  param world World "The world."
  param entity Entity "An entity with an og.MotionAgent."
  param desc WalkGoalDesc "Where, how fast, and the pose on arrival."
  returns Result.Void "Refused for an entity without og.MotionAgent or without drive, or a position that is not finite."
}
```

<sub>`api.ogt` line 27040</sub>
