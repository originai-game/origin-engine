<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`motion`](../motion.md) · [index](../index.md)

<a id="fn-pose"></a>

### `og.motion.pose`

Queue a library pose where the character stands, after its other goals.

Also known as: emote, gesture, wave, idle pose, play a pose

```ts
og.motion.pose(world: World, entity: Entity, desc?: PoseGoalDesc): Result<void>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `entity` | `Entity` | no | An entity with an og.MotionAgent. |
| `desc` | `PoseGoalDesc` | yes | The pose and how long it is current. |

**Returns** `Result<void>`: Refused for an entity without og.MotionAgent or a pose the library lacks (og.motion.poses).

Binding symbol `og_motion_pose` · publish builds: kept

```ogt
fn pose {
  doc "Queue a library pose where the character stands, after its other goals."
  aliases "emote, gesture, wave, idle pose, play a pose"
  param world World "The world."
  param entity Entity "An entity with an og.MotionAgent."
  param desc PoseGoalDesc "The pose and how long it is current."
  returns Result.Void "Refused for an entity without og.MotionAgent or a pose the library lacks (og.motion.poses)."
}
```

<sub>`api.ogt` line 27007</sub>
