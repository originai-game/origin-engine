<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`physics`](../physics.md) · [index](../index.md)

<a id="fn-massProperties"></a>

### `og.physics.massProperties`

The live simulation's computed mass, world centre of mass and local principal inertia for a RigidBody entity, or null if not a body. Uses the same initialization and tick boundary as castRay.

```ts
og.physics.massProperties(world: World, entity: Entity): Result<MassProperties | null>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `entity` | `Entity` | no | The body's entity. |

**Returns** `Result<MassProperties | null>`.

Binding symbol `og_physics_mass_properties` · publish builds: kept

```ogt
fn massProperties {
  doc "The live simulation's computed mass, world centre of mass and local principal inertia for a RigidBody entity, or null if not a body. Uses the same initialization and tick boundary as castRay."
  param world World "The world."
  param entity Entity "The body's entity."
  returns Result.Option.MassProperties
}
```

<sub>Module `physics` line 2458</sub>
