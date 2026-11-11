<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`destruction`](../destruction.md) · [index](../index.md)

<a id="fn-cell2d"></a>

### `og.destruction.cell2d`

The cell of an og.PixelTerrain under a world point, as the last tick left it: 0 empty, 255 bedrock, any other ground (fill2d writes 1). Outside the terrain, 0.

```ts
og.destruction.cell2d(world: World, entity: Entity, point: Vector3): Result<number>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `entity` | `Entity` | no | An entity with og.PixelTerrain. |
| `point` | `Vector3` | no | A world point (its xy). |

**Returns** `Result<number>`: Refused for an entity without og.PixelTerrain, or one og.destruction2d has not yet filled (the tick after it is added).

Binding symbol `og_destruction_cell2d` · publish builds: kept

```ogt
fn cell2d {
  doc "The cell of an og.PixelTerrain under a world point, as the last tick left it: 0 empty, 255 bedrock, any other ground (fill2d writes 1). Outside the terrain, 0."
  param world World "The world."
  param entity Entity "An entity with og.PixelTerrain."
  param point Vector3 "A world point (its xy)."
  returns Result.u32 "Refused for an entity without og.PixelTerrain, or one og.destruction2d has not yet filled (the tick after it is added)."
}
```

<sub>Module `destruction` line 518</sub>
