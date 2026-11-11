<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`destruction`](../destruction.md) · [index](../index.md)

<a id="fn-fill2d"></a>

### `og.destruction.fill2d`

Make a round patch of an og.PixelTerrain solid (ground, or bedrock), this tick, as carve2d. Bedrock stays bedrock.

Also known as: build terrain, add ground, paint terrain

```ts
og.destruction.fill2d(world: World, entity: Entity, desc?: Fill2dDesc): Result<void>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `entity` | `Entity` | no | An entity with og.PixelTerrain. |
| `desc` | `Fill2dDesc` | yes | Where, how big and what with. |

**Returns** `Result<void>`: Refused for an entity without og.PixelTerrain, or a point or radius that is not finite.

Binding symbol `og_destruction_fill2d` · publish builds: kept

```ogt
fn fill2d {
  doc "Make a round patch of an og.PixelTerrain solid (ground, or bedrock), this tick, as carve2d. Bedrock stays bedrock."
  aliases "build terrain, add ground, paint terrain"
  param world World "The world."
  param entity Entity "An entity with og.PixelTerrain."
  param desc Fill2dDesc "Where, how big and what with."
  returns Result.Void "Refused for an entity without og.PixelTerrain, or a point or radius that is not finite."
}
```

<sub>Module `destruction` line 492</sub>
