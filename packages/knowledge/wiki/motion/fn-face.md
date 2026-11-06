<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`motion`](../motion.md) · [index](../index.md)

<a id="fn-face"></a>

### `og.motion.face`

Queue a turn to a heading where the character stands; needs og.MotionAgent.drive.

```ts
og.motion.face(world: World, entity: Entity, desc?: FaceGoalDesc): Result<void>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `entity` | `Entity` | no | An entity with an og.MotionAgent. |
| `desc` | `FaceGoalDesc` | yes | The heading and the least time the goal is current. |

**Returns** `Result<void>`: Refused for an entity without og.MotionAgent or without drive.

Binding symbol `og_motion_face` · publish builds: kept

```ogt
fn face {
  doc "Queue a turn to a heading where the character stands; needs og.MotionAgent.drive."
  param world World "The world."
  param entity Entity "An entity with an og.MotionAgent."
  param desc FaceGoalDesc "The heading and the least time the goal is current."
  returns Result.Void "Refused for an entity without og.MotionAgent or without drive."
}
```

<sub>`api.ogt` line 26981</sub>
