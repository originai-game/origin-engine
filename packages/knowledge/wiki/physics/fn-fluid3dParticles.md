<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`physics`](../physics.md) · [index](../index.md)

<a id="fn-fluid3dParticles"></a>

### `og.physics.fluid3dParticles`

An og.Fluid3D's particle positions after the last step, in world space (emitted particles last, in the order they appeared).

```ts
og.physics.fluid3dParticles(world: World, entity: Entity): Result<readonly Vector3[]>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `entity` | `Entity` | no | The fluid's entity. |

**Returns** `Result<readonly Vector3[]>`: Refused for an entity without an og.Fluid3D.

Binding symbol `og_physics_fluid3d_particles` · publish builds: kept

```ogt
fn fluid3dParticles {
  doc "An og.Fluid3D's particle positions after the last step, in world space (emitted particles last, in the order they appeared)."
  param world World "The world."
  param entity Entity "The fluid's entity."
  returns Result.List.Vector3 "Refused for an entity without an og.Fluid3D."
}
```

<sub>Module `physics` line 2125</sub>
