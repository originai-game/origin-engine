<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`physics`](../physics.md) · [index](../index.md)

<a id="fn-fluidParticles"></a>

### `og.physics.fluidParticles`

An og.Fluid2D's particle positions after the last step, in world space.

```ts
og.physics.fluidParticles(world: World, entity: Entity): Result<readonly Vector2[]>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `entity` | `Entity` | no | The fluid's entity. |

**Returns** `Result<readonly Vector2[]>`: Refused for an entity without an og.Fluid2D.

Binding symbol `og_physics_fluid_particles` · publish builds: kept

```ogt
fn fluidParticles {
  doc "An og.Fluid2D's particle positions after the last step, in world space."
  param world World "The world."
  param entity Entity "The fluid's entity."
  returns Result.List.Vector2 "Refused for an entity without an og.Fluid2D."
}
```

<sub>Module `physics` line 2078</sub>
