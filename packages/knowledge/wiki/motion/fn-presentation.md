<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`motion`](../motion.md) · [index](../index.md)

<a id="fn-presentation"></a>

### `og.motion.presentation`

Presentation-only readiness on both hosts, for a host's cover: ready once generated frames and their palette exist for the current rendered frame (authored fallback does not count), or failed with a reason. Game logic must not branch on it: it depends on GPU timing and is outside snapshots and hashes; a development build warns when a system reads it on the gpu backend.

See `animation §10`.

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
  doc "Presentation-only readiness on both hosts, for a host's cover: ready once generated frames and their palette exist for the current rendered frame (authored fallback does not count), or failed with a reason. Game logic must not branch on it: it depends on GPU timing and is outside snapshots and hashes; a development build warns when a system reads it on the gpu backend."
  guide "animation §10"
  param world World "The world."
  param entity Entity "An entity with an og.MotionAgent."
  returns Result.MotionPresentationStatus "Refused for an entity without og.MotionAgent."
}
```

<sub>Module `motion` line 339</sub>
