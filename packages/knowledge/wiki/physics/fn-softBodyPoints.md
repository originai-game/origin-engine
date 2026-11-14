<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`physics`](../physics.md) · [index](../index.md)

<a id="fn-softBodyPoints"></a>

### `og.physics.softBodyPoints`

An og.SoftBody's points and the triangles of its surface after the last step (none before its first step). The engine draws visible soft bodies itself; this is for game logic (where it touches, how squashed it is) and custom drawing.

```ts
og.physics.softBodyPoints(world: World, entity: Entity): Result<SoftBodyPoints>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `entity` | `Entity` | no | The soft body's entity. |

**Returns** `Result<SoftBodyPoints>`: Refused for an entity without an og.SoftBody.

Binding symbol `og_physics_soft_body_points` · publish builds: kept

```ogt
fn softBodyPoints {
  doc "An og.SoftBody's points and the triangles of its surface after the last step (none before its first step). The engine draws visible soft bodies itself; this is for game logic (where it touches, how squashed it is) and custom drawing."
  param world World "The world."
  param entity Entity "The soft body's entity."
  returns Result.SoftBodyPoints "Refused for an entity without an og.SoftBody."
}
```

<sub>Module `physics` line 2103</sub>
