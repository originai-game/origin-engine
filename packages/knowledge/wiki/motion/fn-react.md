<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`motion`](../motion.md) · [index](../index.md)

<a id="fn-react"></a>

### `og.motion.react`

Replace the authored reaction overlay without discarding locomotion goals. Empty desc releases it. A motion-action asset names a humanoid clip and rig, hold interval and optional head-local hand contact. The hand is solved after the look so it stays on the hood.

```ts
og.motion.react(world: World, entity: Entity, desc?: ReactionDesc): Result<void>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `entity` | `Entity` | no | An entity with an og.MotionAgent and humanoid og.Skeleton. |
| `desc` | `ReactionDesc` | yes | Action, direction, strength and timing. |

**Returns** `Result<void>`: Refused for a missing agent, skeleton, action or required bones, an invalid hold interval, or invalid inputs.

Binding symbol `og_motion_react` · publish builds: kept

```ogt
fn react {
  doc "Replace the authored reaction overlay without discarding locomotion goals. Empty desc releases it. A motion-action asset names a humanoid clip and rig, hold interval and optional head-local hand contact. The hand is solved after the look so it stays on the hood."
  param world World "The world."
  param entity Entity "An entity with an og.MotionAgent and humanoid og.Skeleton."
  param desc ReactionDesc "Action, direction, strength and timing."
  returns Result.Void "Refused for a missing agent, skeleton, action or required bones, an invalid hold interval, or invalid inputs."
}
```

<sub>`api.ogt` line 26929</sub>
