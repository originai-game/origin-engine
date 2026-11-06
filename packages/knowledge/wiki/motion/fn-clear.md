<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`motion`](../motion.md) · [index](../index.md)

<a id="fn-clear"></a>

### `og.motion.clear`

Drop the agent's goals, the current one included: the character holds where it is.

```ts
og.motion.clear(world: World, entity: Entity): Result<void>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `entity` | `Entity` | no | An entity with an og.MotionAgent. |

**Returns** `Result<void>`: Refused for an entity without og.MotionAgent.

Binding symbol `og_motion_clear` · publish builds: kept

```ogt
fn clear {
  doc "Drop the agent's goals, the current one included: the character holds where it is."
  param world World "The world."
  param entity Entity "An entity with an og.MotionAgent."
  returns Result.Void "Refused for an entity without og.MotionAgent."
}
```

<sub>`api.ogt` line 26914</sub>
