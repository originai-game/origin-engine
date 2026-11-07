<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`motion`](../motion.md) · [index](../index.md)

<a id="fn-lookAt"></a>

### `og.motion.lookAt`

Replace the timed look overlay without interrupting locomotion. Empty desc releases it. State is snapshotted and advances only on ticks; the turn is applied after generated motion or the animation fallback, on both hosts.

```ts
og.motion.lookAt(world: World, entity: Entity, desc?: LookAtDesc): Result<void>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `entity` | `Entity` | no | An entity with an og.MotionAgent and humanoid og.Skeleton. |
| `desc` | `LookAtDesc` | yes | Target, limits and timing. |

**Returns** `Result<void>`: Refused for a missing agent, skeleton or mapped head/spine, or invalid inputs.

Binding symbol `og_motion_look_at` · publish builds: kept

```ogt
fn lookAt {
  doc "Replace the timed look overlay without interrupting locomotion. Empty desc releases it. State is snapshotted and advances only on ticks; the turn is applied after generated motion or the animation fallback, on both hosts."
  param world World "The world."
  param entity Entity "An entity with an og.MotionAgent and humanoid og.Skeleton."
  param desc LookAtDesc "Target, limits and timing."
  returns Result.Void "Refused for a missing agent, skeleton or mapped head/spine, or invalid inputs."
}
```

<sub>Module `motion` line 300</sub>
