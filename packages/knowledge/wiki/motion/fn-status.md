<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`motion`](../motion.md) · [index](../index.md)

<a id="fn-status"></a>

### `og.motion.status`

The agent's current goal and queue.

```ts
og.motion.status(world: World, entity: Entity): Result<MotionStatus>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `entity` | `Entity` | no | An entity with an og.MotionAgent. |

**Returns** `Result<MotionStatus>`: Refused for an entity without og.MotionAgent.

Binding symbol `og_motion_status` · publish builds: kept

```ogt
fn status {
  doc "The agent's current goal and queue."
  param world World "The world."
  param entity Entity "An entity with an og.MotionAgent."
  returns Result.MotionStatus "Refused for an entity without og.MotionAgent."
}
```

<sub>`api.ogt` line 27058</sub>
