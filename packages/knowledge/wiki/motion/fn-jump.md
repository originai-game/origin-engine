<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`motion`](../motion.md) · [index](../index.md)

<a id="fn-jump"></a>

### `og.motion.jump`

Queue a jump after the agent's other goals; needs og.MotionAgent.drive. Takeoff, height, flight and landing are the simulation's (the controller's with an og.CharacterController, a fixed ballistic arc without one), deterministic and in snapshots; the motion model only draws it. A jump cleared in the air without a controller holds where it is.

```ts
og.motion.jump(world: World, entity: Entity, desc?: JumpGoalDesc): Result<void>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `entity` | `Entity` | no | An entity with an og.MotionAgent. |
| `desc` | `JumpGoalDesc` | yes | Height, travel, preparation and recovery. |

**Returns** `Result<void>`: Refused for an entity without og.MotionAgent or without drive, a pose the library lacks, or a height, distance, heading or time that is out of range or not finite.

Binding symbol `og_motion_jump` · publish builds: kept

```ogt
fn jump {
  doc "Queue a jump after the agent's other goals; needs og.MotionAgent.drive. Takeoff, height, flight and landing are the simulation's (the controller's with an og.CharacterController, a fixed ballistic arc without one), deterministic and in snapshots; the motion model only draws it. A jump cleared in the air without a controller holds where it is."
  param world World "The world."
  param entity Entity "An entity with an og.MotionAgent."
  param desc JumpGoalDesc "Height, travel, preparation and recovery."
  returns Result.Void "Refused for an entity without og.MotionAgent or without drive, a pose the library lacks, or a height, distance, heading or time that is out of range or not finite."
}
```

<sub>`api.ogt` line 27027</sub>
