<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`physics`](../physics.md) · [index](../index.md)

<a id="fn-ropePoints"></a>

### `og.physics.ropePoints`

An og.Rope's points after the last step, from its entity to its end (empty before its first step). The engine draws visible ropes itself; this is for game logic and custom drawing.

```ts
og.physics.ropePoints(world: World, entity: Entity): Result<readonly Vector3[]>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `entity` | `Entity` | no | The rope's entity. |

**Returns** `Result<readonly Vector3[]>`: Refused for an entity without an og.Rope.

Binding symbol `og_physics_rope_points` · publish builds: kept

```ogt
fn ropePoints {
  doc "An og.Rope's points after the last step, from its entity to its end (empty before its first step). The engine draws visible ropes itself; this is for game logic and custom drawing."
  param world World "The world."
  param entity Entity "The rope's entity."
  returns Result.List.Vector3 "Refused for an entity without an og.Rope."
}
```

<sub>Module `physics` line 2001</sub>
