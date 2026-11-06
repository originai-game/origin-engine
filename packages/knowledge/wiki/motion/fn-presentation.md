<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`motion`](../motion.md) · [index](../index.md)

<a id="fn-presentation"></a>

### `og.motion.presentation`

Presentation-only readiness on both hosts. Game logic must not branch on this value: it depends on GPU timing and is outside snapshots and hashes. A host may keep its page or shell cover up while the agent remains in the world and warms, then lift the cover on ready or failed (showing reason). Never write a component, gate a goal or advance a tick from this answer. ready requires generated frames and their palette for the current rendered frame; authored fallback does not count. Available in the probe's dump_motion too.

```ts
og.motion.presentation(world: World, entity: Entity): Result<MotionPresentationStatus>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `entity` | `Entity` | no | An entity with an og.MotionAgent. |

**Returns** `Result<MotionPresentationStatus>`: Refused for an entity without og.MotionAgent.

Binding symbol `og_motion_presentation` · publish builds: kept

```ogt
fn presentation {
  doc "Presentation-only readiness on both hosts. Game logic must not branch on this value: it depends on GPU timing and is outside snapshots and hashes. A host may keep its page or shell cover up while the agent remains in the world and warms, then lift the cover on ready or failed (showing reason). Never write a component, gate a goal or advance a tick from this answer. ready requires generated frames and their palette for the current rendered frame; authored fallback does not count. Available in the probe's dump_motion too."
  param world World "The world."
  param entity Entity "An entity with an og.MotionAgent."
  returns Result.MotionPresentationStatus "Refused for an entity without og.MotionAgent."
}
```

<sub>`api.ogt` line 27042</sub>
